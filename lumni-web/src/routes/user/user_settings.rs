use leptos::ev::SubmitEvent;
use leptos::logging::log;
use leptos::*;

use crate::components::forms::builders::{
    ElementBuilder, FormBuilder, FormType, LoadParameters, SubmitParameters,
};
use crate::components::forms::input::{perform_validation, FieldContentType};
use crate::components::forms::{
    ConfigurationFormMeta, FormData, FormStorageHandler, LocalStorageWrapper,
};
use crate::GlobalState;

const FORM_ID: &str = "user_settings";

#[derive(Debug, PartialEq, Clone)]
pub struct RouteParams {
    id: String,
}

#[component]
pub fn UserSettings() -> impl IntoView {
    let vault = use_context::<RwSignal<GlobalState>>()
        .expect("state to have been provided")
        .with(|state| state.vault.clone())
        .expect("vault to have been initialized");

    let username = "admin".to_string(); // TODO: get this from vault

    let form_id = FORM_ID;

    let is_loading = create_rw_signal(false);
    let load_error = create_rw_signal(None::<String>);

    let vault_clone = vault.clone();
    let handle_load = move |form_data_rw: RwSignal<Option<FormData>>| {
        let storage_wrapper = LocalStorageWrapper::new(vault_clone.clone());
        let storage_handler = FormStorageHandler::new(storage_wrapper);

        is_loading.set(true);
        spawn_local(async move {
            // match load_config_from_vault(&vault, form_id).await {
            match storage_handler.load_config(form_id).await {
                Ok(Some(config)) => {
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

    let is_submitting = create_rw_signal(false);
    let submit_error = create_rw_signal(None::<String>);
    let handle_submit = move |ev: SubmitEvent, form_data: Option<FormData>| {
        ev.prevent_default();
        is_submitting.set(true);

        let storage_wrapper = LocalStorageWrapper::new(vault.clone());
        let storage_handler = FormStorageHandler::new(storage_wrapper);

        spawn_local(async move {
            if let Some(form_data) = form_data {
                let form_elements = form_data.elements();
                let validation_errors = perform_validation(&form_elements);
                if validation_errors.is_empty() {
                    let result = storage_handler.save_config(&form_data).await;
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

    let load_parameters = LoadParameters::new(Some(Box::new(handle_load)));

    let submit_parameters = SubmitParameters::new(
        Box::new(handle_submit),
        Some(is_submitting),
        Some(submit_error),
        None,
    );

    let form_meta = ConfigurationFormMeta::with_id(&form_id.to_string());
    let mut form = FormBuilder::new(
        &username,
        form_meta,
        FormType::LoadAndSubmitData(load_parameters, submit_parameters),
    );

    form.add_element(
        ElementBuilder::new("field1", FieldContentType::PlainText)
            .with_label("a"),
    )
    .add_element(
        ElementBuilder::new("field2", FieldContentType::PlainText)
            .with_label("b"),
    );

    let form = form.build(None);
    form.to_view()
}
