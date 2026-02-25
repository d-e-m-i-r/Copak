use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
        // use hash to avoid String Cloning
        let mut last_hash: u64 = 0;
        if let Ok(initial_clipboard_content) = app.clipboard().read_text() {
            if !initial_clipboard_content.is_empty() {
                last_hash = get_clipboard_content_hash(&initial_clipboard_content);
            }
        }
        // continuously monitor clipboard content changes
        loop {
            match app.clipboard().read_text() {
                Ok(current_clipboard_content) => {
                    let mut current_hash: u64 = 0;
                    if !current_clipboard_content.is_empty() {
                        current_hash = get_clipboard_content_hash(&current_clipboard_content);
                    }
                    if current_clipboard_content.len() > 0 {
                        if current_hash != last_hash {
                            println!("Clipboard content changed: {}", current_clipboard_content);
                            last_hash = current_hash;
                        };
                    } else {
                        // If the clipboard is empty, we can consider it as a change
                        if last_hash != 0 {
                            println!("Clipboard content cleared");
                            last_hash = 0; // Reset hash for empty content
                        }
                    }
                } //debugging purpose
                Err(e) => {
                    if last_hash != 0 {
                        eprintln!("Failed to read clipboard content: {}", e);
                        last_hash = 0; // Reset hash on error to detect future changes
                    }
                }
            }
            thread::sleep(Duration::from_secs(1)); // Check every second
        }
    });
}

fn get_clipboard_content_hash(clipboard_content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    clipboard_content.hash(&mut hasher);
    hasher.finish()
}
