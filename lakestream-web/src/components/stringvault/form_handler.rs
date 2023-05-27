use std::collections::HashMap;

use async_trait::async_trait;
use leptos::html::Div;
use leptos::*;
use wasm_bindgen_futures::spawn_local;

use super::{FormView, FormOwner, InputData, SecureStringError, StringVault};

#[async_trait(?Send)]
pub trait ConfigManager: Clone {
    fn get_default_config(&self) -> HashMap<String, String>;
    fn default_fields(&self) -> HashMap<String, InputData>;
    fn tag(&self) -> String;
    fn id(&self) -> String;
}

pub struct FormHandler<T: ConfigManager + Clone + 'static> {
    config_manager: T,
    vault: StringVault,
}

impl<T: ConfigManager + Clone + 'static> FormHandler<T> {
    pub fn new(config_manager: T, vault: StringVault) -> Self {
        Self {
            config_manager,
            vault,
        }
    }

    fn form_owner(&self) -> FormOwner {
        FormOwner {
            tag: self.config_manager.tag().to_uppercase(),
            id: self.config_manager.id(),
        }
    }

    pub fn form_data_handler(&self, cx: Scope) -> HtmlElement<Div> {
        let (loaded_config, set_loaded_config) = create_signal(cx, None);
        let (load_config_error, set_load_config_error) =
            create_signal(cx, None::<String>);

        let vault_clone = self.vault.clone();
        let config_manager_clone = self.config_manager.clone();
        let form_owner_clone = self.form_owner();

        create_effect(cx, move |_| {
            let vault_clone = vault_clone.clone();
            let default_config = config_manager_clone.get_default_config();
            let form_owner = form_owner_clone.clone();
            spawn_local(async move {
                match vault_clone.load_secure_configuration(form_owner).await {
                    Ok(new_config) => {
                        log!("loading config: {:?}", new_config);
                        set_loaded_config(Some(new_config));
                    }
                    Err(e) => match e {
                        SecureStringError::PasswordNotFound(_)
                        | SecureStringError::NoLocalStorageData => {
                            // use default if cant load existing
                            log!("Cant load existing configuration: {:?}", e);
                            set_loaded_config(Some(default_config));
                        }
                        _ => {
                            log!("error loading config: {:?}", e);
                            set_load_config_error(Some(e.to_string()));
                        }
                    },
                };
            });
        });

        let vault_clone = self.vault.clone();
        let config_manager_clone = self.config_manager.clone();
        let default_config = config_manager_clone.default_fields();
        let form_owner_clone = self.form_owner();
        view! { cx,
            <div>
            {move ||
                if let Some(loaded_config) = loaded_config.get() {

                    view! {
                        cx,
                        <div>
                        <FormView
                            vault={vault_clone.clone()}
                            form_owner={form_owner_clone.clone()}
                            initial_config={loaded_config}
                            default_config={default_config.clone()}
                        />
                        </div>
                    }
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