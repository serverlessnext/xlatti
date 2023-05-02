use std::collections::HashMap;

use log::error;

use super::bucket::{configure_bucket_url, S3Bucket, S3Credentials};
use super::client::{S3Client, S3ClientConfig};
use super::parse_http_response::{
    extract_continuation_token, parse_bucket_objects, parse_file_objects,
};
use super::request_handler::http_get_with_redirect_handling;
use crate::base::config::Config;
use crate::base::interfaces::ObjectStoreTrait;
use crate::http::requests::http_get_request;
use crate::{
    FileObject, FileObjectFilter, FileObjectVec, LakestreamError, ObjectStore,
    AWS_MAX_LIST_OBJECTS,
};

pub struct ListFilesParams<'a> {
    s3_bucket: &'a S3Bucket,
    prefix: Option<String>,
    max_keys: Option<u32>,
    s3_client: &'a mut S3Client,
    continuation_token: Option<String>,
    recursive: bool,
    filter: &'a Option<FileObjectFilter>,
}

pub async fn list_files(
    s3_bucket: &S3Bucket,
    prefix: Option<&str>,
    recursive: bool,
    max_keys: Option<u32>,
    filter: &Option<FileObjectFilter>,
    file_objects: &mut FileObjectVec,
) -> Result<(), LakestreamError> {
    let mut s3_client =
        create_s3_client(s3_bucket.config(), Some(s3_bucket.name()));

    list_files_next(
        &mut ListFilesParams {
            s3_bucket,
            prefix: prefix.map(|p| p.to_owned()),
            max_keys,
            s3_client: &mut s3_client,
            continuation_token: None, // start with no continuation_token
            recursive,
            filter: &(*filter).clone(),
        },
        file_objects,
    )
    .await?;
    Ok(())
}

async fn list_files_next(
    params: &mut ListFilesParams<'_>,
    file_objects: &mut FileObjectVec,
) -> Result<(), LakestreamError> {
    let mut directory_stack = std::collections::VecDeque::new();
    let mut temp_file_objects = Vec::new();

    directory_stack.push_back(params.prefix.clone());

    let effective_max_keys =
        get_effective_max_keys(params.filter, params.max_keys);

    while let Some(prefix) = directory_stack.pop_front() {
        let mut virtual_directories = Vec::<String>::new();
        loop {
            let (response_body, updated_s3_client) =
                http_get_with_redirect_handling(
                    params.s3_bucket.config(),
                    params.s3_bucket.name(),
                    params.s3_client,
                    prefix.as_deref(),
                    Some(effective_max_keys),
                    params.continuation_token.as_deref(),
                )
                .await?;
            if let Some(new_s3_client) = updated_s3_client {
                *params.s3_client = new_s3_client;
            }

            params.continuation_token = process_response_body(
                &response_body,
                params.recursive,
                params.filter,
                &mut temp_file_objects,
                &mut virtual_directories,
            );

            if params.continuation_token.is_none()
                || file_objects.len()
                    >= params.max_keys.unwrap_or(AWS_MAX_LIST_OBJECTS) as usize
            {
                break;
            }
        }

        // Extend file_objects with temp_file_objects and clear temp_file_objects
        file_objects.extend_async(temp_file_objects.drain(..)).await;

        if params.recursive {
            for virtual_directory in virtual_directories.drain(..) {
                if file_objects.len()
                    == params.max_keys.unwrap_or(AWS_MAX_LIST_OBJECTS) as usize
                {
                    break;
                }

                directory_stack.push_back(Some(virtual_directory));
            }
        }

        params.continuation_token = None;
    }

    Ok(())
}

fn process_file_object(
    file_object: FileObject,
    recursive: bool,
    filter: &Option<FileObjectFilter>,
    virtual_directories: &mut Vec<String>,
    temp_file_objects: &mut Vec<FileObject>,
) {
    if file_object.name().ends_with('/') {
        if recursive {
            virtual_directories.push(file_object.name().to_owned());
        }
        if filter.is_none() {
            temp_file_objects.push(file_object);
        }
    } else {
        if let Some(ref filter) = filter {
            if !filter.matches(&file_object) {
                return;
            }
        }
        temp_file_objects.push(file_object);
    }
}

fn process_response_body(
    response_body: &str,
    recursive: bool,
    filter: &Option<FileObjectFilter>,
    temp_file_objects: &mut Vec<FileObject>,
    virtual_directories: &mut Vec<String>,
) -> Option<String> {
    if !response_body.is_empty() {
        let file_objects_list =
            parse_file_objects(response_body).unwrap_or_default();

        for file_object in file_objects_list {
            process_file_object(
                file_object,
                recursive,
                filter,
                virtual_directories,
                temp_file_objects,
            );
        }

        extract_continuation_token(response_body)
    } else {
        None
    }
}

pub async fn list_buckets(
    config: &Config,
) -> Result<Vec<ObjectStore>, LakestreamError> {
    let mut s3_client = create_s3_client(config, None);

    let headers: HashMap<String, String> =
        s3_client.generate_list_buckets_headers().unwrap();
    let result = http_get_request(&s3_client.url().clone(), &headers).await;

    let bucket_objects = match result {
        Ok((body, _)) => {
            match parse_bucket_objects(&body, Some(config.clone())) {
                Ok(bucket_objects) => bucket_objects,
                Err(e) => {
                    error!("Error listing bucket objects: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            error!("Error in http_get_request: {}", e);
            Vec::new()
        }
    };

    Ok(bucket_objects)
}

fn create_s3_client(config: &Config, bucket_name: Option<&str>) -> S3Client {
    let region = config
        .get("AWS_REGION")
        .expect("Missing region in the configuration");
    let access_key = config
        .get("AWS_ACCESS_KEY_ID")
        .expect("Missing access_key in the configuration");
    let secret_key = config
        .get("AWS_SECRET_ACCESS_KEY")
        .expect("Missing secret_key in the configuration");

    let credentials =
        S3Credentials::new(String::from(access_key), String::from(secret_key));
    let endpoint_url =
        config.settings.get("S3_ENDPOINT_URL").map(String::as_str);
    let bucket_url = configure_bucket_url(region, endpoint_url, bucket_name);

    let s3_client_config =
        S3ClientConfig::new(credentials, &bucket_url, region);
    S3Client::new(s3_client_config)
}

fn get_effective_max_keys(
    filter: &Option<FileObjectFilter>,
    max_keys: Option<u32>,
) -> u32 {
    if filter.is_some() {
        AWS_MAX_LIST_OBJECTS
    } else {
        max_keys.unwrap_or(AWS_MAX_LIST_OBJECTS)
    }
}
