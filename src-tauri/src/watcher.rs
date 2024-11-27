use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, ResourceExt},
    Client,
};
use tauri::{AppHandle, Emitter, EventTarget};

pub fn start(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || {
        tauri::async_runtime::block_on(async move {
            let client = if let Ok(client) = Client::try_default().await {
                client
            } else {
                let _ = handle
                    .emit_to(EventTarget::app(), "watcher-error", "")
                    .expect("failed to emit watcher event");
                return;
            };
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
                std::thread::sleep(std::time::Duration::from_secs(5));
                let _ = handle
                    .emit_to(EventTarget::app(), "watcher", 0)
                    .expect("failed to emit watcher event");
            }
        });
    });
    Ok(())
}
