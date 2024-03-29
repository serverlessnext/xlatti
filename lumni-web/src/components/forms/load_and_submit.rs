use std::rc::Rc;

use leptos::ev::SubmitEvent;
use leptos::*;

use super::html_form::{Form, HtmlForm};
use super::submit_handler::{CustomSubmitHandler, SubmitFormHandler};
use super::view_handler::ViewHandler;
use super::{FormData, SubmitInput};
use crate::components::buttons::{ButtonType, FormButton};
use crate::components::forms::builders::{LoadParameters, SubmitParameters};

pub struct LoadAndSubmitForm {
    form: HtmlForm,
    view_handler: ViewHandler,
    form_button: Option<FormButton>,
}

impl LoadAndSubmitForm {
    pub fn new(
        form: HtmlForm,
        load_parameters: LoadParameters,
        submit_parameters: SubmitParameters,
    ) -> Self {
        let is_processing = submit_parameters
            .is_submitting()
            .unwrap_or_else(|| create_rw_signal(false));
        let process_error = submit_parameters
            .validation_error()
            .unwrap_or_else(|| create_rw_signal(None));

        if let Some(load_handler) = load_parameters.load_handler {
            // load handler writes to form_data_rw
            load_handler(form.form_data_rw());
        }

        let form_button = submit_parameters.form_button;

        let form_data_rw = form.form_data_rw();
        let function = submit_parameters.submit_handler;
        let custom_submit_handler = Box::new(CustomSubmitHandler::new(
            form_data_rw,
            Rc::new(
                move |ev: SubmitEvent, _submit_input: Option<SubmitInput>| {
                    function(ev, form_data_rw.get());
                },
            ),
            is_processing,
            process_error,
        ));

        let form_handler =
            Rc::new(SubmitFormHandler::new(custom_submit_handler));
        let view_handler = ViewHandler::new(form_handler);

        Self {
            form,
            view_handler,
            form_button,
        }
    }

    pub fn to_view(&self) -> View {
        let form_button = self.form_button.clone().unwrap_or(
            FormButton::new(ButtonType::Submit, None).set_enabled(false),
        );
        self.view_handler.to_view(Some(form_button))
    }
}

impl Form for LoadAndSubmitForm {
    fn form_data_rw(&self) -> RwSignal<Option<FormData>> {
        self.form.form_data_rw()
    }

    fn to_view(&self) -> View {
        self.to_view()
    }
}
