use std::collections::HashMap;

use leptos::html::Div;
use leptos::*;
use leptos_router::{use_params, Params, ParamsError, ParamsMap};

use crate::components::forms::object_store::ObjectStore;
use crate::components::object_store_list::ObjectStoreList;
use crate::components::stringvault::FormHandler;
use crate::GlobalState;

#[derive(Debug, PartialEq, Clone)]
pub struct RouteParams {
    id: String,
}

impl Params for RouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        let id = map
            .get("id")
            .ok_or_else(|| ParamsError::MissingParam("id".to_string()))?;
        Ok(Self { id: id.to_string() })
    }
}

#[component]
pub fn ObjectStoresId(cx: Scope) -> impl IntoView {
    let vault = use_context::<RwSignal<GlobalState>>(cx)
        .expect("state to have been provided")
        .with(|state| state.vault.clone())
        .expect("vault to have been initialized");

    let params = use_params::<RouteParams>(cx);
    let id: Option<String> = match params.try_get() {
        Some(Ok(route_params)) => Some(route_params.id.clone()),
        Some(Err(_)) => None,
        None => None,
    };

    let name = if let Some(id) = &id {
        let valid_map: HashMap<String, String> =
            ObjectStoreList::load_from_local_storage()
                .into_iter()
                .map(|item| (item.id(), item.name))
                .collect();
        valid_map.get(id).cloned()
    } else {
        None
    };

    let form_data_handler: HtmlElement<Div> = match name {
        Some(name) => {
            let config_manager = ObjectStore::new(name);
            let form_handler = FormHandler::new(config_manager, vault);
            form_handler.form_data_handler(cx)
        }
        _ => {
            // Render 404 page
            view! {
                cx,
                <div>
                    <h1>"404: Page Not Found"</h1>
                    <p>"The page you requested could not be found."</p>
                </div>
            }
            .into()
        }
    };

    view! {
        cx,
        <div>
            {form_data_handler}
        </div>
    }
}
