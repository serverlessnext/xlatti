use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use crate::localfs::backend::LocalFsBucket;
use crate::s3::backend::S3Bucket;
use crate::{
    EnvironmentConfig, FileObjectFilter, LakestreamError, TableCallback
};
use crate::table::{Table, FileObjectTable};

#[derive(Debug, Clone)]
pub enum ObjectStore {
    S3Bucket(S3Bucket),
    LocalFsBucket(LocalFsBucket),
}

impl ObjectStore {
    pub fn new(
        name: &str,
        config: EnvironmentConfig,
    ) -> Result<ObjectStore, String> {
        if name.starts_with("s3://") {
            let name = name.trim_start_matches("s3://");
            let bucket =
                S3Bucket::new(name, config).map_err(|err| err.to_string())?;
            Ok(ObjectStore::S3Bucket(bucket))
        } else if name.starts_with("localfs://") {
            let name = name.trim_start_matches("localfs://");
            let local_fs = LocalFsBucket::new(name, config)
                .map_err(|err| err.to_string())?;
            Ok(ObjectStore::LocalFsBucket(local_fs))
        } else {
            Err("Unsupported object store.".to_string())
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ObjectStore::S3Bucket(bucket) => bucket.name(),
            ObjectStore::LocalFsBucket(local_fs) => local_fs.name(),
        }
    }

    pub fn config(&self) -> &EnvironmentConfig {
        match self {
            ObjectStore::S3Bucket(bucket) => bucket.config(),
            ObjectStore::LocalFsBucket(local_fs) => local_fs.config(),
        }
    }

    pub fn uri(&self) -> String {
        match self {
            ObjectStore::S3Bucket(bucket) => {
                format!("s3://{}", bucket.name())
            }
            ObjectStore::LocalFsBucket(local_fs) => {
                format!("{}", local_fs.name())
            }
        }
    }

    pub async fn list_files(
        &self,
        prefix: Option<&str>,
        recursive: bool,
        max_keys: Option<u32>,
        filter: &Option<FileObjectFilter>,
    ) -> Result<Box<dyn Table>, LakestreamError> { 
        let mut table = FileObjectTable::new();
        match self {
            ObjectStore::S3Bucket(bucket) => {
                bucket
                    .list_files(
                        prefix,
                        recursive,
                        max_keys,
                        filter,
                        &mut table,
                    )
                    .await
            }
            ObjectStore::LocalFsBucket(local_fs) => {
                local_fs
                    .list_files(
                        prefix,
                        recursive,
                        max_keys,
                        filter,
                        &mut table,
                    )
                    .await
            }
        }?;
        Ok(Box::new(table))
    }

    pub async fn list_files_with_callback(
        &self,
        prefix: Option<&str>,
        recursive: bool,
        max_files: Option<u32>,
        filter: &Option<FileObjectFilter>,
        callback: Arc<dyn TableCallback>,
    ) -> Result<(), LakestreamError> {

        let mut table = FileObjectTable::new();
        table.set_callback(callback);
    
        match self {
            ObjectStore::S3Bucket(bucket) => {
                bucket
                    .list_files(
                        prefix,
                        recursive,
                        max_files,
                        filter,
                        &mut table,
                    )
                    .await
            }
            ObjectStore::LocalFsBucket(local_fs) => {
                local_fs
                    .list_files(
                        prefix,
                        recursive,
                        max_files,
                        filter,
                        &mut table,
                    )
                    .await
            }
        }
    }

    pub async fn get_object(
        &self,
        key: &str,
        data: &mut Vec<u8>,
    ) -> Result<(), LakestreamError> {
        match self {
            ObjectStore::S3Bucket(bucket) => bucket.get_object(key, data).await,
            ObjectStore::LocalFsBucket(local_fs) => {
                local_fs.get_object(key, data).await
            }
        }
    }
}

#[async_trait(?Send)]
pub trait ObjectStoreTrait: Send {
    fn name(&self) -> &str;
    fn config(&self) -> &EnvironmentConfig;
    async fn list_files(
        &self,
        prefix: Option<&str>,
        recursive: bool,
        max_keys: Option<u32>,
        filter: &Option<FileObjectFilter>,
        table: &mut FileObjectTable,
    ) -> Result<(), LakestreamError>;
    async fn get_object(
        &self,
        key: &str,
        data: &mut Vec<u8>,
    ) -> Result<(), LakestreamError>;
    async fn head_object(
        &self,
        key: &str,
    ) -> Result<(u16, HashMap<String, String>), LakestreamError>;
}

