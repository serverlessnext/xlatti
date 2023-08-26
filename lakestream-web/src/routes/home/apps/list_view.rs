use std::collections::HashMap;

use leptos::html::Input;
use leptos::*;
use localencrypt::{ItemMetaData, LocalStorage};

use crate::external::builders::{ConfigTemplate, load_app_config};
use crate::components::forms::FormError;
use crate::GlobalState;

// TODO: currently we have only 1 App, in future updates we need to
// make this dynamic
const APP_NAME: &str = "objectstore-s3";

#[component]
pub fn ConfigurationListView(cx: Scope) -> impl IntoView {
    let vault = use_context::<RwSignal<GlobalState>>(cx)
        .expect("state to have been provided")
        .with(|state| state.vault.clone())
        .expect("vault to have been initialized");

    let local_storage = match vault.backend() {
        localencrypt::StorageBackend::Browser(browser_storage) => {
            browser_storage
                .local_storage()
                .unwrap_or_else(|| panic!("Invalid browser storage type"))
        }
        _ => panic!("Invalid storage backend"),
    };

    let selected_template = create_rw_signal(cx, APP_NAME.to_string());
    let (is_loading, set_is_loading) = create_signal(cx, true);
    let (item_list, set_item_list) =
        create_signal(cx, ConfigurationList::new(local_storage));
    provide_context(cx, set_item_list);

    let input_ref = create_node_ref::<Input>(cx);

    // TODO: implement error handling
    let (_, set_submit_error) = create_signal(cx, None::<String>);

    fn get_input_value(input_ref: NodeRef<Input>) -> Option<String> {
        let input = input_ref.get()?;
        let value = input.value().trim().to_string();
        if !value.is_empty() {
            input.set_value("");
            Some(value)
        } else {
            None
        }
    }

    create_effect(cx, move |_| {
        spawn_local({
            let local_storage = match vault.backend() {
                localencrypt::StorageBackend::Browser(browser_storage) => {
                    browser_storage.local_storage().unwrap_or_else(|| {
                        panic!("Invalid browser storage type")
                    })
                }
                _ => panic!("Invalid storage backend"),
            };

            async move {
                let object_store_list = item_list.get_untracked();
                let initial_items = object_store_list
                    .load_from_vault()
                    .await
                    .unwrap_or_default();
                set_item_list.set(ConfigurationList {
                    items: initial_items,
                    local_storage,
                });
                set_is_loading.set(false);
            }
        });
    });

    create_effect(cx, move |_| {
        if let Some(input) = input_ref.get() {
            request_animation_frame(move || {
                let _ = input.focus();
            });
        }
    });

    view! {
        cx,
        {move || if is_loading.get() {
            view! { cx, <div>"Loading..."</div> }
        } else {
            view! {
                cx,
                <div>
                <div>
                    <select on:change=move |ev| selected_template.set(event_target_value(&ev)) >
                        <option value=APP_NAME>{APP_NAME}</option>
                    </select>
                    <input class="px-4 py-2"
                        placeholder="Bucket URI"
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                if let Some(name) = get_input_value(input_ref) {
                                    let template = selected_template.get();
                                    let config = load_app_config(&template, name, None);
                                    let item = Box::new(config) as Box<dyn ConfigTemplate>;
                                    set_item_list.update(|item_list| item_list.add(item, set_is_loading, set_submit_error));
                                }
                            }
                        }
                        node_ref=input_ref
                    />
                    <button class="px-4 py-2" on:click=move |_| {
                        if let Some(name) = get_input_value(input_ref) {
                            let template = selected_template.get();
                            let config =  load_app_config(&template, name, None);
                            let item = Box::new(config) as Box<dyn ConfigTemplate>;
                            set_item_list.update(|item_list| item_list.add(item, set_is_loading, set_submit_error));
                        }
                    }> "Add Item" </button>
                </div>
                <div>
                    <ul>
                        <For
                            each={move || item_list.get().items}
                            key=|item| item.id()
                            view=move |cx, item| view! { cx, <ListItem item set_is_loading/> }
                        />
                    </ul>
                </div>
                </div>
            }
        }}
    }
}

#[component]
fn ListItem(
    cx: Scope,
    item: Box<dyn ConfigTemplate>,
    set_is_loading: WriteSignal<bool>,
) -> impl IntoView {
    let set_item = use_context::<WriteSignal<ConfigurationList>>(cx).unwrap();
    let item_id = item.id();
    let item_name = item.name();

    view! { cx,
        <li>
            <div class="px-4 py-2">
                {item_name.clone()} " | "
                <a href={format!("/apps/{}/{}", APP_NAME, item_id)}>
                    "Form"
                </a>
                " | "
                <a href={format!("/apps/{}/{}?view=TextArea", APP_NAME, item_id)}>
                    "TextArea"
                </a>
                " | "
                <button class="text-red-500 hover:text-red-700" on:click=move |_| set_item.update(|t| t.remove(item_id.clone(), set_is_loading))> "delete" </button>
            </div>
        </li>
    }
}

pub struct ConfigurationList {
    pub items: Vec<Box<dyn ConfigTemplate>>,
    pub local_storage: LocalStorage,
}

impl Clone for ConfigurationList {
    fn clone(&self) -> Self {
        ConfigurationList {
            items: self.items.iter().map(|item| item.clone_box()).collect(),
            local_storage: self.local_storage.clone(),
        }
    }
}

impl ConfigurationList {
    pub fn new(local_storage: LocalStorage) -> Self {
        Self {
            items: vec![],
            local_storage,
        }
    }

    pub async fn load_from_vault(
        &self,
    ) -> Result<Vec<Box<dyn ConfigTemplate>>, FormError> {
        let configs = self
            .local_storage
            .list_items()
            .await
            .map_err(FormError::from)?;

        let items: Result<Vec<_>, _> = configs
            .into_iter()
            .map(|form_data| {
                let config_name = form_data.tags().and_then(|tags| {
                    tags.get("ConfigName")
                        .cloned()
                        .or_else(|| Some("Untitled".to_string()))
                });

                let app_name = form_data.tags().and_then(|tags| tags.get("AppName").cloned());

                app_name.and_then(|app_name| {
                    config_name.map(|name| {
                        log!("Loaded name {} with template {}", name, app_name);
                        let config =
                            load_app_config(&app_name, name, Some(form_data.id()));
                        Box::new(config) as Box<dyn ConfigTemplate>
                    })
                })
                .ok_or_else(|| {
                    FormError::SubmitError(
                        "Form name not found".to_string(),
                    )
                })
            })
            .collect();
        items
    }

    pub fn add(
        &mut self,
        item: Box<dyn ConfigTemplate>,
        set_is_submitting: WriteSignal<bool>,
        _set_submit_error: WriteSignal<Option<String>>,
    ) {
        set_is_submitting.set(true);

        let name = item.name();
        let id = item.id();
        let app_name = item.app_name();

        let mut tags = HashMap::new();
        tags.insert("ConfigName".to_string(), name);
        tags.insert("AppName".to_string(), app_name);
        let meta_data = ItemMetaData::new_with_tags(&id, tags);

        spawn_local({
            let mut local_storage = self.local_storage.clone();
            async move {
                let _ = local_storage.add_item(meta_data).await;
                set_is_submitting.set(false);
            }
        });
        self.items.push(item);
    }

    pub fn remove(
        &mut self,
        item_id: String,
        set_is_loading: WriteSignal<bool>,
    ) {
        set_is_loading.set(true);
        spawn_local({
            let item_id = item_id.clone();
            let mut local_storage = self.local_storage.clone();
            async move {
                let _ = local_storage.delete_item(&item_id).await;
                set_is_loading.set(false);
            }
        });

        self.items.retain(|item| item.id() != item_id);
    }
}