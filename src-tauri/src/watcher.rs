use std::collections::BTreeMap;
use std::sync::Mutex;

use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, ResourceExt},
    config::{Config, KubeConfigOptions, Kubeconfig},
    Client,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use tokio::time;

use crate::settings::SettingsStore;

// Pod status
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PodStatus {
    /// Pod name
    pub name: String,
    container_name: Option<String>,
    status: String,
    labels: BTreeMap<String, String>,
    pub workspace_folder: Option<String>,
}

// Running pods status
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClusterStatus {
    context: String,
    namespace: String,
    pub pods: Vec<PodStatus>,
}

pub fn start(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let kubeconfig = Kubeconfig::read()?;
    let current_context = kubeconfig
        .current_context
        .clone()
        .ok_or("no current context")?;
    let client = tauri::async_runtime::block_on(async move {
        let config = Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
            .await
            .map_err(|_| "failed to load kubeconfig")?;
        return Client::try_from(config).map_err(|_| "failed to load config for k8s");
    })?;
    println!("default context: {}", current_context);
    tauri::async_runtime::spawn(async move {
        let namespace = client.default_namespace().to_string();
        let api: Api<Pod> = Api::default_namespaced(client);
        loop {
            match api.list(&ListParams::default()).await {
                Ok(pod_list) => {
                    let mut pods: Vec<PodStatus> = Vec::new();
                    for pod in pod_list {
                        let name = pod.name_any();
                        let container_name = pod.spec.map(|s| s.containers[0].name.clone());
                        let labels = pod.metadata.labels.unwrap_or_default();
                        let workspace_folder = container_name.clone().and_then(|container_name| {
                            resolve_workspace_folder(
                                &handle,
                                &current_context,
                                &namespace,
                                &container_name,
                                &labels,
                            )
                        });
                        pods.push(PodStatus {
                            name,
                            container_name,
                            status: pod
                                .status
                                .and_then(|s| s.container_statuses)
                                .and_then(|cs| cs[0].state.clone())
                                .map_or("Unknown".to_string(), |s| {
                                    if s.running.is_some() {
                                        "Running".to_string()
                                    } else if s.waiting.is_some() {
                                        "Waiting".to_string()
                                    } else if s.terminated.is_some() {
                                        "Terminated".to_string()
                                    } else {
                                        "Unknown".to_string()
                                    }
                                }),
                            labels: labels.clone(),
                            workspace_folder,
                        });
                    }
                    let status = ClusterStatus {
                        context: current_context.clone(),
                        namespace: namespace.clone(),
                        pods,
                    };
                    let _ = handle
                        .emit_to(EventTarget::app(), "watcher", status)
                        .expect("failed to emit watcher event");
                }
                Err(e) => {
                    println!("failed to list pods: {}", e);
                }
            }
            tokio::time::sleep(time::Duration::from_secs(5)).await;
        }
    });
    Ok(())
}

fn resolve_workspace_folder(
    handle: &AppHandle,
    context: &str,
    namespace: &str,
    container_name: &str,
    labels: &BTreeMap<String, String>,
) -> Option<String> {
    let settings_store = handle.state::<Mutex<SettingsStore>>();
    let settings_store = settings_store.lock().unwrap();
    let settings = settings_store.app_settings();
    return settings.workspaces.iter().find_map(|ws| {
        if ws.context == context && ws.namespace == namespace && ws.container_name == container_name
        {
            let satisfied = ws
                .labels
                .iter()
                .all(|(k, v)| labels.get(k).map(|val| val == v).unwrap_or(false));
            if satisfied {
                return Some(ws.workspace_folder.clone());
            }
        }
        return None;
    });
}
