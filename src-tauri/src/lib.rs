use tauri::{
    include_image,
    menu::{IconMenuItem, Menu, MenuBuilder, MenuItem, NativeIcon},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Error, Listener, Wry,
};
use watcher::{ClusterStatus, PodStatus};

mod watcher;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
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
                // TODO: resolve favorite pods
                match handle.tray_by_id("hugill-tray") {
                    Some(tray) => {
                        let _ =
                            tray.set_menu(get_tray_menu(&handle, Some(status.clone().pods)).ok());
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
