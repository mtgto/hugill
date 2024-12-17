use std::collections::HashMap;
use std::sync::Mutex;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::json;
use settings::SettingsStore;
use tauri::{
    async_runtime::JoinHandle,
    include_image,
    menu::{IconMenuItem, Menu, MenuBuilder, MenuItem, NativeIcon},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
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
    poll_interval_msec: u64,
    workspaces: Vec<WorkspaceSetting>,
}

struct AppStatus {
    watcher_join_handle: Option<JoinHandle<()>>,
    tray_opened: bool,
}

#[tauri::command]
fn start_cluster_watcher(app_handle: tauri::AppHandle) -> Result<(), String> {
    stop_cluster_watcher(app_handle.clone());
    let poll_interval_msec = app_handle
        .state::<Mutex<SettingsStore>>()
        .lock()
        .unwrap()
        .app_settings()
        .poll_interval_msec;
    watcher::start(app_handle.clone(), poll_interval_msec).map_or_else(
        |e| {
            println!("Failed to start watcher: {e}");
            Err(format!("Failed to setup containers watcher: {e}"))
        },
        |join_handle| {
            app_handle
                .state::<Mutex<AppStatus>>()
                .lock()
                .unwrap()
                .watcher_join_handle = Some(join_handle);
            println!("Watcher started");
            Ok(())
        },
    )
}

#[tauri::command]
fn stop_cluster_watcher(app_handle: tauri::AppHandle) {
    let state = app_handle.state::<Mutex<AppStatus>>();
    let mut state = state.lock().unwrap();
    if let Some(watcher_join_handle) = &state.watcher_join_handle {
        watcher_join_handle.abort();
        state.watcher_join_handle = None;
    }
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
    let remote_uri = format!("vscode-remote://{encoded}{workspace_folder}");
    let shell = app_handle.shell();
    let output = tauri::async_runtime::block_on(async move {
        shell
            .command("code")
            .args(["--folder-uri", &remote_uri])
            .output()
            .await
    });
    match output {
        Err(e) => {
            // ex: code command not found
            println!("Failed to open remote container: {e}");
            Err(format!("Failed to open remote container: {e}"))
        }
        Ok(output) => {
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
                    false
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
                Ok(())
            } else {
                println!("Exit with code: {}", output.status.code().unwrap());
                Err("Failed to open remote container".to_string())
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            start_cluster_watcher,
            stop_cluster_watcher,
            open_remote_container
        ])
        .setup(|app| {
            let default_workspace_settings: [WorkspaceSetting; 0] = [];
            let store: SettingsStore = app
                .store_builder("settings.json")
                .default("workspaces", json!(default_workspace_settings))
                .build()?
                .into();
            app.manage(Mutex::new(store));
            app.manage(Mutex::new(AppStatus {
                watcher_join_handle: None,
                tray_opened: false,
            }));
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
                    pod_id => {
                        let cluster = handle.state::<ClusterStatus>();
                        cluster
                            .pods
                            .iter()
                            .find(|pod| pod.name == pod_id)
                            .and_then(|pod| {
                                pod.workspace_folder.clone().and_then(|workspace_folder| {
                                    pod.container_name.clone().and_then(|container_name| {
                                        let mut labels = HashMap::new();
                                        for (key, value) in pod.labels.iter() {
                                            labels.insert(key.clone(), value.clone());
                                        }
                                        open_remote_container(
                                            handle.clone(),
                                            &cluster.context,
                                            &cluster.namespace,
                                            &pod.name,
                                            &container_name,
                                            labels,
                                            &workspace_folder,
                                        )
                                        .ok()
                                    })
                                })
                            });
                        println!("other menu event");
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Down,
                        ..
                    } => {
                        // Tray is opened
                        tray.app_handle()
                            .state::<Mutex<AppStatus>>()
                            .lock()
                            .unwrap()
                            .tray_opened = true;
                    }
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        // Tray is closed
                        tray.app_handle()
                            .state::<Mutex<AppStatus>>()
                            .lock()
                            .unwrap()
                            .tray_opened = false;
                    }
                    TrayIconEvent::Leave { .. } => {
                        // Tray is closed
                        tray.app_handle()
                            .state::<Mutex<AppStatus>>()
                            .lock()
                            .unwrap()
                            .tray_opened = false;
                    }
                    _ => (),
                })
                .build(app);
            let handle = app.handle().clone();
            let _ = app.listen("watcher", move |event| {
                let status: ClusterStatus = serde_json::from_str(event.payload()).unwrap();
                // Filter pods which has no workspace_folder setting
                let pods: Vec<PodStatus> = status
                    .clone()
                    .pods
                    .into_iter()
                    .filter(|pod| pod.workspace_folder.is_some())
                    .collect();
                handle.manage(status.clone());
                if let Some(tray) = handle.tray_by_id("hugill-tray") {
                    if !handle
                        .state::<Mutex<AppStatus>>()
                        .lock()
                        .unwrap()
                        .tray_opened
                    {
                        let _ = tray.set_menu(get_tray_menu(&handle, Some(pods)).ok());
                    }
                }
                handle
                    .emit_to("hugill", "cluster-status", status.clone())
                    .expect("failed to emit updated status");
                println!("watcher event received: {:?}", status);
            });
            let handle = app.handle().clone();
            let _ = app.listen("watcher-error", move |event| {
                // failed to receive cluster status
                let message: String = serde_json::from_str(event.payload()).unwrap();
                handle
                    .emit_to("hugill", "cluster-status-error", message)
                    .expect("failed to emit watcher error event");
                println!("watcher error event received: {}", event.payload());
                if let Some(tray) = handle.tray_by_id("hugill-tray") {
                    if !handle
                        .state::<Mutex<AppStatus>>()
                        .lock()
                        .unwrap()
                        .tray_opened
                    {
                        let _ = tray.set_menu(get_tray_menu(&handle, None).ok());
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_tray_menu(handle: &AppHandle, pods: Option<Vec<PodStatus>>) -> Result<Menu<Wry>, Error> {
    let builder = MenuBuilder::new(handle);
    match pods {
        Some(pods) => {
            let mut builder = builder;
            for pod in pods {
                let native_icon = match pod.status.as_str() {
                    "Running" => NativeIcon::StatusAvailable,
                    "Waiting" => NativeIcon::StatusPartiallyAvailable,
                    "Terminated" => NativeIcon::StatusUnavailable,
                    _ => NativeIcon::StatusNone,
                };
                builder = builder.item(&IconMenuItem::with_id_and_native_icon(
                    handle,
                    &pod.name,
                    &pod.name,
                    true,
                    Some(native_icon),
                    None::<&str>,
                )?);
            }
            builder.separator()
        }
        None => builder,
    }
    .item(&MenuItem::with_id(
        handle,
        "quit",
        "Quit",
        true,
        None::<&str>,
    )?)
    .build()
}
