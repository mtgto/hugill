use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, ResourceExt},
    config::{Config, KubeConfigOptions, Kubeconfig},
    Client,
};
use tauri::{AppHandle, Emitter, EventTarget};
use tokio::time;

pub fn start(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (current_context, client) = tauri::async_runtime::block_on(async move {
        let kubeconfig = Kubeconfig::read()?;
        let current_context = kubeconfig
            .current_context
            .clone()
            .ok_or("no current context")?;
        let config =
            Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default()).await?;
        let client = Client::try_from(config)?;
        return Ok::<(std::string::String, Client), Box<dyn std::error::Error>>((
            current_context,
            client,
        ));
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
