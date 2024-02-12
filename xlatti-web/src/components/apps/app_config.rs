use std::fmt::Debug;

use uuid::Uuid;

use super::get_app_handler;
use super::parse_config::{parse_yaml, ConfigYamlType};
use crate::api::error::Error;
use crate::api::handler::AppHandler;
use crate::components::forms::builders::ElementBuilder;

pub struct AppConfig {
    app_uri: String,
    handler: Box<dyn AppHandler>,
    profile_name: String,
    profile_id: String,
}

impl Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppConfig")
            .field("app_uri", &self.app_uri)
            .field("profile_name", &self.profile_name)
            .field("profile_id", &self.profile_id)
            .finish()
    }
}

impl Clone for AppConfig {
    fn clone(&self) -> Self {
        AppConfig {
            app_uri: self.app_uri.clone(),
            handler: self.handler.clone_box(),
            profile_name: self.profile_name.clone(),
            profile_id: self.profile_id.clone(),
        }
    }
}

impl AppConfig {
    pub fn new<S: Into<String>>(
        app_uri: S,
        profile_name: S,
        profile_id: Option<S>,
    ) -> AppConfig {
        let app_uri = app_uri.into();
        let handler = get_app_handler(&app_uri).unwrap();
        // TODO: handle None

        AppConfig {
            app_uri,
            handler,
            profile_name: profile_name.into(),
            profile_id: profile_id
                .map_or_else(|| Uuid::new_v4().to_string(), |id| id.into()),
        }
    }

    fn load_config(&self) -> &str {
        self.handler.load_config()
    }

    pub fn handler(&self) -> &dyn AppHandler {
        self.handler.as_ref()
    }

    pub fn profile_name(&self) -> String {
        self.profile_name.clone()
    }

    pub fn profile_id(&self) -> String {
        self.profile_id.clone()
    }

    pub fn app_uri(&self) -> String {
        self.app_uri.clone()
    }

    pub fn configuration_form_elements(
        &self,
    ) -> Result<Vec<ElementBuilder>, Error> {
        parse_yaml(self.load_config(), ConfigYamlType::ConfigurationEnvironment)
    }

    pub fn interface_form_elements(
        &self,
    ) -> Result<Vec<ElementBuilder>, Error> {
        parse_yaml(self.load_config(), ConfigYamlType::InterfaceForm)
    }
}