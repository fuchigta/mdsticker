// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use uuid::Uuid;

#[tauri::command]
fn save_sticker(window: tauri::Window, markdown: &str) {
    print!("{0} {1}", window.label(), markdown)
}

#[tauri::command]
fn new_sticker(handle: tauri::AppHandle) {
    let label = Uuid::new_v4();

    let w = tauri::WindowBuilder::new(
        &handle,
        label,
        tauri::WindowUrl::App("index.html".into()),
    )
    .build()
    .unwrap();

    w.set_title("mdsticker");
}

#[tauri::command]
fn remove_sticker(window: tauri::Window) {
    print!("{0} removed", window.label());
    window.close();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_sticker,
            new_sticker,
            remove_sticker
        ])
        .setup(|app| {
            new_sticker(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
