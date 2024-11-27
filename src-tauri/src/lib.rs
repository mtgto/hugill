use tauri::{
    include_image,
    menu::{IconMenuItem, Menu, MenuItem, NativeIcon, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Listener,
};

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
            let running_i = IconMenuItem::with_id_and_native_icon(
                app,
                "running",
                "Running",
                true,
                Some(NativeIcon::StatusAvailable),
                None::<&str>,
            )?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&running_i, &PredefinedMenuItem::separator(app)?, &quit_i],
            )?;
            let _ = TrayIconBuilder::with_id("hugill-tray")
                .tooltip("Hugill")
                .icon(include_image!("./icons/SystemTray@2x.png"))
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app);
            let _ = app.listen("watcher", move |event| {
                println!("watcher event received");
            });
            let _ = app.listen("watcher-error", move |event| {
                // failed to create kube client
                // TODO: restart watcher
                println!("watcher error event received");
            });
            watcher::start(app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
