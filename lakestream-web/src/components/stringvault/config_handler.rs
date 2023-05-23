use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use async_trait::async_trait;
use leptos::ev::SubmitEvent;
use leptos::html::{Div, Input};
use leptos::*;
use wasm_bindgen_futures::spawn_local;

use super::{StringVault, SecureStringError};

#[async_trait(?Send)]
pub trait ConfigManager: Clone {
    fn get_default_config(&self) -> HashMap<String, String>;
    fn id(&self) -> String;
}

pub struct ConfigFormView<T: ConfigManager + Clone + 'static> {
    config_manager: T,
    vault: Rc<RefCell<StringVault>>,
}

impl<T: ConfigManager + Clone + 'static> ConfigFormView<T> {
    pub fn new(config_manager: T, vault: Rc<RefCell<StringVault>>) -> Self {
        Self {
            config_manager,
            vault,
        }
    }

    pub fn form_data_handler(&self, cx: Scope) -> HtmlElement<Div> {
        let (loaded_config, set_loaded_config) = create_signal(cx, None);
        let (load_config_error, set_load_config_error) =
            create_signal(cx, None::<String>);

        let vault_clone = self.vault.clone();
        let config_manager_clone = self.config_manager.clone();

        create_effect(cx, move |_| {
            let vault_clone = vault_clone.clone();
            let id_string = config_manager_clone.id();
            let default_config = config_manager_clone.get_default_config();
            spawn_local(async move {
                let vault = vault_clone.borrow();
                match vault.load_secure_configuration(&id_string).await {
                    Ok(new_config) => {
                        set_loaded_config(Some(new_config));
                    }
                    Err(e) => match e {
                        SecureStringError::PasswordNotFound(_) | SecureStringError::NoLocalStorageData => {
                            // use default if cant load existing
                            log!("Cant load existing configuration: {:?}", e);
                            set_loaded_config(Some(default_config));
                        },
                        _ => {
                            log!("error loading config: {:?}", e);
                            set_load_config_error(Some(e.to_string()));
                        }
                    }
                };
            });
        });

        let vault_clone = self.vault.clone();
        let uuid = self.config_manager.id();
        view! { cx,
            <div>
            {move ||
                if let Some(loaded_config) = loaded_config.get() {
                    form_view(cx, vault_clone.clone(), uuid.clone(), &loaded_config)
                }
                else if let Some(error) = load_config_error.get() {
                    view! {
                        cx,
                        <div>
                            {"Error loading configuration: "}
                            {error}
                        </div>
                    }
                }
                else {
                    view! {
                        cx,
                        <div>
                            "Loading..."
                        </div>
                    }
                }
            }
            </div>
        }
    }
}

fn form_view(
    cx: Scope,
    vault: Rc<RefCell<StringVault>>,
    uuid: String,
    loaded_config: &HashMap<String, String>,
) -> HtmlElement<Div> {
    let input_elements = create_input_elements(cx, loaded_config);
    let input_elements_clone_submit = input_elements.clone();

    let on_submit: Rc<RefCell<dyn OnSubmit>> = Rc::new(RefCell::new(
        move |ev: SubmitEvent,
              input_elements: HashMap<String, NodeRef<Input>>| {
            handle_form_submission(
                ev,
                vault.clone(),
                uuid.clone(),
                input_elements,
            );
        },
    ));

    view! {
        cx,
        <div>
        <form class="flex flex-wrap w-96" on:submit=move |ev| {on_submit.borrow_mut().call(ev, input_elements_clone_submit.clone())}>
            {loaded_config.iter().map(move |(key, initial_value)| {
                let input_elements_clone_map = input_elements.clone();
                let input_ref = input_elements_clone_map.get(key).expect("input ref to exist").clone();
                create_input_field_view(cx, key, initial_value, input_ref)
            }).collect::<Vec<_>>()}
            <button
                type="submit"
                class="bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
            >
                "Save"
            </button>
        </form>
        </div>
    }
}

pub trait OnSubmit {
    fn call(
        &mut self,
        ev: SubmitEvent,
        input_elements: HashMap<String, NodeRef<Input>>,
    );
}

impl<F: FnMut(SubmitEvent, HashMap<String, NodeRef<Input>>)> OnSubmit for F {
    fn call(
        &mut self,
        ev: SubmitEvent,
        input_elements: HashMap<String, NodeRef<Input>>,
    ) {
        self(ev, input_elements)
    }
}

fn create_input_elements(
    cx: Scope,
    updated_config: &HashMap<String, String>,
) -> HashMap<String, NodeRef<Input>> {
    let mut input_elements: HashMap<String, NodeRef<Input>> = HashMap::new();
    for (key, _value) in updated_config {
        input_elements.insert(key.clone(), create_node_ref(cx));
    }
    input_elements
}

fn handle_form_submission(
    ev: SubmitEvent,
    vault: Rc<RefCell<StringVault>>,
    uuid: String,
    input_elements: HashMap<String, NodeRef<Input>>,
) {
    ev.prevent_default();

    let mut config: HashMap<String, String> = HashMap::new();
    for (key, input_ref) in &input_elements {
        let value = input_ref().expect("input to exist").value();
        config.insert(key.clone(), value);
    }

    spawn_local(async move {
        let mut vault = vault.borrow_mut();
        match vault.save_secure_configuration(&uuid, config.clone()).await {
            Ok(_) => {
                log!("Successfully saved secure configuration");
            }
            Err(e) => {
                log!("Failed to save secure configuration. Error: {:?}", e);
            }
        };
    });
    log!("Saved items");
}

fn create_input_field_view(
    cx: Scope,
    key: &String,
    initial_value: &String,
    input_ref: NodeRef<Input>,
) -> HtmlElement<Div> {
    view! { cx,
        <div class="bg-blue-200 w-full flex-col items-start text-left mb-4">
            <label class="text-left px-2 w-full">{format!("{} ", key)}</label>
            <input
                type="text"
                value=initial_value
                class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                node_ref=input_ref
            />
        </div>
    }
}