use std::sync::Arc;

use serde_json::json;
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
        let namespace = self
            .store
            .get("namespace")
            .and_then(|namespace| serde_json::from_value::<String>(namespace).ok());
        let poll_interval_msec = self
            .store
            .get("poll_interval_msec")
            .and_then(|poll_interval_msec| serde_json::from_value::<u64>(poll_interval_msec).ok());
        let workspaces = self.store.get("workspaces");
        let workspaces = workspaces.and_then(|workspace_settings| {
            serde_json::from_value::<Vec<WorkspaceSetting>>(workspace_settings).ok()
        });
        AppSettings {
            namespace,
            poll_interval_msec: poll_interval_msec.unwrap_or(5000),
            workspaces: workspaces.unwrap_or_default(),
        }
    }

    pub fn update_workspaces(&self, workspaces: Vec<WorkspaceSetting>) {
        self.store.set("workspaces", json!(workspaces));
    }
}
