use tauri::{AppHandle, Emitter, EventTarget};

pub fn start(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = handle
            .emit_to(EventTarget::app(), "watcher", 0)
            .expect("failed to emit watcher event");
    });
    Ok(())
}
