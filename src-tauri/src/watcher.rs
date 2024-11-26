use tauri::{AppHandle, Emitter, EventTarget};

pub fn start(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || {
        handle.emit_to(EventTarget::app(), "watcher", 0);
        /*
        loop {
            thread::sleep(std::time::Duration::from_secs(5));
            app.emit_to(EventTarget::app(), "watcher", []).expect("failed to emit watcher event");
        }
        */
    });
    Ok(())
}
