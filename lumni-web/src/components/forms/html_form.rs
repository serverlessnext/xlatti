use leptos::*;

use crate::components::forms::builders::FormType;
use crate::components::forms::input::FormElement;
use crate::components::forms::{
    ConfigurationFormMeta, FormData, FormViewOptions, LoadAndSubmitForm,
    LoadForm, SubmitForm,
};

pub struct HtmlForm {
    html_form_meta: HtmlFormMeta,
    pub elements: Vec<FormElement>,
    form_data_rw: RwSignal<Option<FormData>>,
}

impl HtmlForm {
    pub fn new(
        name: &str,
        form_meta: ConfigurationFormMeta,
        view_options: Option<FormViewOptions>,
        elements: Vec<FormElement>,
    ) -> Self {
        let html_form_meta = HtmlFormMeta::new(name, &form_meta.id());

        let form_data_rw = if !elements.is_empty() {
            let form_data = FormData::build(form_meta, &elements, view_options);
            create_rw_signal(Some(form_data))
        } else {
            create_rw_signal(None)
        };

        Self {
            html_form_meta,
            elements,
            form_data_rw,
        }
    }

    pub fn build(self, form_type: FormType) -> Box<dyn Form> {
        match form_type {
            FormType::SubmitData(submit_parameters) => {
                Box::new(SubmitForm::new(self, submit_parameters))
            }
            FormType::LoadData(load_parameters) => {
                Box::new(LoadForm::new(self, Some(load_parameters)))
            }
            FormType::LoadElements => Box::new(LoadForm::new(self, None)),
            FormType::LoadAndSubmitData(load_parameters, submit_parameters) => {
                Box::new(LoadAndSubmitForm::new(
                    self,
                    load_parameters,
                    submit_parameters,
                ))
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.html_form_meta.name
    }

    pub fn id(&self) -> &str {
        &self.html_form_meta.id
    }

    pub fn form_data_rw(&self) -> RwSignal<Option<FormData>> {
        self.form_data_rw
    }
}

pub trait Form {
    fn form_data_rw(&self) -> RwSignal<Option<FormData>>;
    fn to_view(&self) -> View;
}

#[derive(Clone, Debug)]
pub struct HtmlFormMeta {
    name: String,
    id: String,
}

impl HtmlFormMeta {
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
        }
    }
}
