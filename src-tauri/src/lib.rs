use std::collections::HashMap;
use std::sync::Mutex;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::json;
use settings::SettingsStore;
use tauri::{
    include_image,
    menu::{IconMenuItem, Menu, MenuBuilder, MenuItem, NativeIcon},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Error, Listener, Manager, Wry,
};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;
use watcher::{ClusterStatus, PodStatus};

mod settings;
mod watcher;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WorkspaceSetting {
    context: String,
    namespace: String,
    container_name: String,
    workspace_folder: String,
    labels: HashMap<String, String>,
}

pub struct AppSettings {
    workspaces: Vec<WorkspaceSetting>,
}

#[tauri::command]
fn open_remote_container(
    app_handle: tauri::AppHandle,
    context: &str,
    namespace: &str,
    pod_name: &str,
    container_name: &str,
    labels: HashMap<String, String>,
    workspace_folder: &str,
) -> Result<(), String> {
    let s = format!("k8s-container+context={context}+podname={pod_name}+namespace={namespace}+name={container_name}");
    let encoded = utf8_percent_encode(&s, NON_ALPHANUMERIC).to_string();
    println!("encoded: {encoded}");
    let remote_uri = format!("vscode-remote://{encoded}{workspace_folder}");
    println!("remote_uri: {remote_uri}");
    let shell = app_handle.shell();
    let output = tauri::async_runtime::block_on(async move {
        return shell
            .command("code")
            .args(["--folder-uri", &remote_uri])
            .output()
            .await
            .unwrap();
    });
    if output.status.success() {
        let settings_store = app_handle.state::<Mutex<SettingsStore>>();
        let settings_store = settings_store.lock().unwrap();
        let mut workspaces = settings_store.app_settings().workspaces;
        let index = workspaces.iter().position(|ws| {
            if ws.context == context
                && ws.namespace == namespace
                && ws.container_name == container_name
            {
                let satisfied = ws
                    .labels
                    .iter()
                    .all(|(k, v)| labels.get(k).map(|val| val == v).unwrap_or(false));
                if satisfied {
                    return true;
                }
            }
            return false;
        });
        match index {
            Some(i) => {
                let ws = &mut workspaces[i];
                ws.workspace_folder = workspace_folder.to_string();
            }
            None => {
                let mut filtered_labels: HashMap<String, String> = HashMap::new();
                for (key, value) in labels.iter() {
                    // ignore hash-related labels (e.g. "pod-template-hash")
                    if !key.ends_with("-hash") {
                        filtered_labels.insert(key.to_string(), value.to_string());
                    }
                }
                workspaces.push(WorkspaceSetting {
                    context: context.to_string(),
                    namespace: namespace.to_string(),
                    container_name: container_name.to_string(),
                    workspace_folder: workspace_folder.to_string(),
                    labels: filtered_labels,
                });
                println!("Added workspace folder for {container_name}");
            }
        }
        settings_store.update_workspaces(workspaces);
        return Ok(());
    } else {
        println!("Exit with code: {}", output.status.code().unwrap());
        return Err("Failed to open remote container".to_string());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![open_remote_container])
        .setup(|app| {
            let default_workspace_settings: [WorkspaceSetting; 0] = [];
            let store: SettingsStore = app
                .store_builder("settings.json")
                .default("workspaces", json!(default_workspace_settings))
                .build()?
                .into();
            app.manage(Mutex::new(store));
            let handle = app.handle().clone();
            let _ = TrayIconBuilder::with_id("hugill-tray")
                .tooltip("Hugill")
                .icon(include_image!("./icons/SystemTray@2x.png"))
                .icon_as_template(true)
                .menu(&get_tray_menu(app.handle(), None)?)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {
                        println!("other menu event");
                    }
                })
                .build(app);
            let _ = app.listen("watcher", move |event| {
                let status: ClusterStatus = serde_json::from_str(event.payload()).unwrap();
                // Filter pods which has no workspace_folder setting
                let pods: Vec<PodStatus> = status
                    .clone()
                    .pods
                    .into_iter()
                    .filter(|pod| pod.workspace_folder.is_some())
                    .collect();
                match handle.tray_by_id("hugill-tray") {
                    Some(tray) => {
                        let _ = tray.set_menu(get_tray_menu(&handle, Some(pods)).ok());
                    }
                    None => (),
                }
                handle
                    .emit_to("hugill", "cluster-status", status.clone())
                    .expect("failed to emit updated status");
                println!("watcher event received: {:?}", status);
            });
            let _ = app.listen("watcher-error", move |event| {
                // failed to create kube client
                // TODO: restart watcher
                println!("watcher error event received: {:?}", event.payload());
            });
            watcher::start(app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_tray_menu(handle: &AppHandle, pods: Option<Vec<PodStatus>>) -> Result<Menu<Wry>, Error> {
    let builder = MenuBuilder::new(handle);
    let builder = match pods {
        Some(pods) => {
            let mut builder = builder;
            for pod in pods {
                builder = builder.item(&IconMenuItem::with_id_and_native_icon(
                    handle,
                    &pod.name,
                    &pod.name,
                    true,
                    Some(NativeIcon::StatusAvailable),
                    None::<&str>,
                )?);
            }
            builder.separator()
        }
        None => builder,
    };
    let builder = builder.item(&MenuItem::with_id(
        handle,
        "quit",
        "Quit",
        true,
        None::<&str>,
    )?);
    return builder.build();
}
