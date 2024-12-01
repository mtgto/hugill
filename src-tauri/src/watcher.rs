use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, ResourceExt},
    config::{Config, KubeConfigOptions, Kubeconfig},
    Client,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, EventTarget};
use tokio::time;

// Pod status
#[derive(Serialize, Deserialize, Debug)]
pub struct PodStatus {
    name: String,
    status: String,
}

// Running pods status
#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    context: String,
    namespace: String,
    pods: Vec<PodStatus>,
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
        println!("watching pods in namespace {}", namespace);
        let pods: Api<Pod> = Api::default_namespaced(client);
        loop {
            match pods.list(&ListParams::default()).await {
                Ok(pods) => {
                    for p in pods {
                        println!("found pod {}", p.name_any());
                        match p.spec {
                            Some(spec) => {
                                println!("container name: {:?}", spec.containers[0].name);
                            }
                            None => {
                                println!("no spec");
                            }
                        };
                    }
                }
                Err(e) => {
                    println!("failed to list pods: {}", e);
                }
            }
            tokio::time::sleep(time::Duration::from_secs(5)).await;
            let _ = handle
                .emit_to(EventTarget::app(), "watcher", 0)
                .expect("failed to emit watcher event");
        }
    });
    Ok(())
}
