use std::thread;
use std::time::Duration;
use tauri::Manager; // for method app.handle()
use tauri_plugin_clipboard_manager::ClipboardExt; // for method  handle.clipboard().read_text()

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            monitor_clipboard(app_handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn monitor_clipboard(app: tauri::AppHandle) {
    thread::spawn(move || {
        //current value of clipboard content
        let mut last_clipboard_content = app.clipboard().read_text().unwrap_or_default();
        loop {
            match app.clipboard().read_text() {
                Ok(current_clipboard_content) => {
                    if current_clipboard_content != last_clipboard_content {
                        println!("Clipboard content changed: {}", current_clipboard_content);
                        last_clipboard_content = current_clipboard_content;
                    }
                } //debugging purpose
                Err(e) => {
                    if !last_clipboard_content.is_empty() {
                        eprintln!("Failed to read clipboard content: {}", e);
                        last_clipboard_content.clear(); // Clear the last content to avoid repeated errors
                    }
                }
            }
            thread::sleep(Duration::from_secs(1)); // Check every second
        }
    });
}
