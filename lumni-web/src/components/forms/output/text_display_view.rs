use leptos::*;

use crate::components::forms::input::{DisplayValue, FormElementState};

#[component]
pub fn TextDisplayView(form_element_state: FormElementState) -> impl IntoView {
    let value_signal = form_element_state.display_value;
    let error_signal = form_element_state.display_error;
    let input_field_data = form_element_state.schema;

    let label_text = input_field_data
        .field_label
        .as_ref()
        .map_or_else(String::new, |label| label.text());

    let initial_value = match value_signal.get_untracked() {
        DisplayValue::Text(text) => text,
    };

    let display_value_signal = create_rw_signal(initial_value);

    view! {
        <div class="w-full flex-col items-start text-left mb-2 p-2 bg-white text-gray-800">
            <InputFieldLabelView
                label_text
            />
            <TextAreaView
                display_value_signal
            />
            <InputFieldErrorView error_signal/>
        </div>
    }
}

#[component]
pub fn InputFieldLabelView(label_text: String) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center">
            <label for="field_id" class="text-base font-semibold text-gray-900">{label_text}</label>
        </div>
    }
}

fn get_input_class(is_enabled: bool) -> &'static str {
    if is_enabled {
        "bg-gray-50 border border-gray-300 text-gray-900 rounded-lg \
         focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5"
    } else {
        "bg-gray-50 border border-gray-300 text-gray-900 rounded-lg block \
         w-full p-2.5"
    }
}

#[component]
pub fn TextAreaView(display_value_signal: RwSignal<String>) -> impl IntoView {
    view! {
        <textarea
            prop:value= { display_value_signal }
            placeholder="none".to_string()
            class={ get_input_class(false) }
            disabled=true
        />
    }
}

#[component]
pub fn InputFieldErrorView(
    error_signal: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="text-red-500">
            { move || error_signal.get().unwrap_or("".to_string()) }
        </div>
    }
    .into_view()
}
