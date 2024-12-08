use std::sync::Arc;
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::{AppSettings, WorkspaceSetting};

pub struct SettingsStore {
    store: Arc<Store<Wry>>,
}

impl From<Arc<Store<Wry>>> for SettingsStore {
    fn from(store: Arc<Store<Wry>>) -> Self {
        Self { store }
    }
}

impl SettingsStore {
    pub fn app_settings(&self) -> AppSettings {
        let workspaces = self.store.get("workspaces");
        let workspaces = workspaces.and_then(|workspace_settings| {
            serde_json::from_value::<Vec<WorkspaceSetting>>(workspace_settings).ok()
        });
        return AppSettings {
            workspaces: workspaces.unwrap_or_default(),
        };
    }
}
