use std::collections::HashMap;

use leptos::ev::SubmitEvent;
use leptos::*;

use super::field_builder::FieldBuilderTrait;
use crate::components::buttons::FormButton;
use crate::components::form_input::FormElement;
use crate::components::forms::{Form, FormData, FormType, HtmlForm};

pub struct FormBuilder {
    title: String,
    id: String,
    elements: Vec<Box<dyn FieldBuilderTrait>>,
    submit_parameters: Option<FormSubmitParameters>,
}

impl FormBuilder {
    pub fn new<S: Into<String>>(title: S, id: S) -> Self {
        Self {
            title: title.into(),
            id: id.into(),
            elements: Vec::new(),
            submit_parameters: None,
        }
    }

    pub fn add_element(mut self, element: Box<dyn FieldBuilderTrait>) -> Self {
        self.elements.push(element);
        self
    }

    pub fn with_submit_parameters(mut self, parameters: FormSubmitParameters) -> Self {
        self.submit_parameters = Some(parameters);
        self
    }

    pub fn build(self, cx: Scope) -> Box<dyn Form> {
        let elements: Vec<FormElement> =
            self.elements.iter().map(|b| b.build()).collect();

        if let Some(submit_parameters) = self.submit_parameters {
            HtmlForm::new(cx, &self.title, &self.id, elements, Some(submit_parameters))
                .build(FormType::Submit)
        } else {
            HtmlForm::new(cx, &self.title, &self.id, elements, None)
                .build(FormType::Load)
        }
    }
}

pub struct FormSubmitParameters {
    // pub parameters are meant to be consumed when used in a form
    pub submit_handler: Box<dyn Fn(SubmitEvent, Option<FormData>)>,
    pub form_button: Option<FormButton>,
    is_submitting: Option<RwSignal<bool>>,
    validation_error: Option<RwSignal<Option<String>>>,
}

impl FormSubmitParameters {
    pub fn new(
        submit_handler: Box<dyn Fn(SubmitEvent, Option<FormData>)>,
        is_submitting: Option<RwSignal<bool>>,
        validation_error: Option<RwSignal<Option<String>>>,
        form_button: Option<FormButton>,
    ) -> Self {
        Self {
            submit_handler,
            form_button,
            is_submitting,
            validation_error,
        }
    }

    pub fn is_submitting(&self) -> Option<RwSignal<bool>> {
        self.is_submitting
    }

    pub fn validation_error(&self) -> Option<RwSignal<Option<String>>> {
        self.validation_error
    }
}
