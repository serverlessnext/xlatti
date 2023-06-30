use leptos::ev::SubmitEvent;
use leptos::*;
use localencrypt::LocalEncrypt;

use super::config_list::{Config, ConfigList};
use super::templates::{ConfigTemplate, Environment, ObjectStoreS3};
use crate::builders::{
    FormBuilder, FormType, LoadParameters, SubmitParameters,
};
use crate::components::form_input::perform_validation;
use crate::components::forms::{FormData, ConfigurationFormMeta, load_config_from_vault, save_config_to_vault};

#[component]
pub fn ConfigurationView(
    cx: Scope,
    vault: LocalEncrypt,
    form_meta: ConfigurationFormMeta,
) -> impl IntoView {
    let is_loading = create_rw_signal(cx, false);
    let load_error = create_rw_signal(cx, None::<String>);

    let vault_clone = vault.clone();
    let form_id_clone = form_meta.form_id.clone();

    let handle_load = move |form_data_rw: RwSignal<Option<FormData>>| {
        let vault = vault_clone.to_owned();
        let form_id = form_id_clone.to_owned();
        is_loading.set(true);
        spawn_local(async move {
            match load_config_from_vault(&vault, &form_id).await {
                Ok(Some(config)) => {
                    log!("Data loaded for form_id: {}", form_id);
                    let mut form_data = form_data_rw.get_untracked().unwrap();
                    form_data.update_with_config(config);
                    form_data_rw.set(Some(form_data));
                    is_loading.set(false);
                }
                Ok(None) => {
                    log!("No data found for form_id: {}", form_id);
                    is_loading.set(false);
                }
                Err(e) => {
                    log!(
                        "Error loading data: {:?} for form_id: {}",
                        e,
                        form_id
                    );
                    load_error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    };

    // Handle form submit
    let is_submitting = create_rw_signal(cx, false);
    let submit_error = create_rw_signal(cx, None::<String>);
    let handle_submit = move |ev: SubmitEvent, form_data: Option<FormData>| {
        ev.prevent_default();
        is_submitting.set(true);
        let vault = vault.to_owned();

        spawn_local(async move {
            if let Some(form_data) = form_data {
                let form_state = form_data.form_state().to_owned();
                let validation_errors = perform_validation(&form_state);
                if validation_errors.is_empty() {
                    let result = save_config_to_vault(&vault, &form_data).await;
                    match result {
                        Ok(_) => {
                            log!("Data submitted successfully");
                            is_submitting.set(false);
                        }
                        Err(e) => {
                            log!("Data submission failed: {:?}", e);
                            submit_error.set(Some(e.to_string()));
                        }
                    }
                } else {
                    log!("Form data is invalid");
                    log!("Validation errors: {:?}", validation_errors);
                }
            } else {
                log!("Form data is empty");
            }
            is_submitting.set(false);
        });
    };

    let load_parameters = LoadParameters::new(
        Some(Box::new(handle_load)),
        Some(is_loading),
        Some(load_error),
    );

    let submit_parameters = SubmitParameters::new(
        Box::new(handle_submit),
        Some(is_submitting),
        Some(submit_error),
        None,
    );

    // Use predefined form elements based on config_type
    let template_name = form_meta.template_name;
    let config_name = form_meta.config_name;
    let form_elements = match template_name.as_str() {
        "ObjectStoreS3" => {
            Config::ObjectStoreS3(ObjectStoreS3::new(&config_name))
        }
        _ => Config::Environment(Environment::new(&config_name)),
    }
    .form_elements(&template_name);

    let form_id = form_meta.form_id;
    let form = FormBuilder::new(
        &config_name,
        &form_id,
        FormType::LoadAndSubmitData(load_parameters, submit_parameters),
    )
    .with_form_elements(form_elements)
    .build(cx);

    form.to_view()
}
