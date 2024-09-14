// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod repository;

use std::path::PathBuf;

use repository::list_stickers;
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use tauri::{
    async_runtime::block_on, AppHandle, CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent
};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Sticker {
    uuid: String,
    markdown: String,
    pos_x: i32,
    pos_y: i32,
    height: u32,
    width: u32,
    pinned: bool,
}

impl Sticker {
    pub fn new(uuid: &str, pos_x: i32, pos_y: i32, height: u32, width: u32) -> Self {
        Sticker {
            uuid: uuid.to_string(),
            markdown: "".to_string(),
            pos_x,
            pos_y,
            height,
            width,
            pinned: false,
        }
    }
}

#[tauri::command]
async fn toggle_sticker_pinned(
    pool: State<'_, sqlx::SqlitePool>,
    window: tauri::Window,
) -> Result<(), String> {
    let pinned = repository::toggle_sticker_pinned(&pool, window.label())
        .await
        .map_err(|e| e.to_string())?;
    let _ = window.set_always_on_top(pinned);

    Ok(())
}

#[tauri::command]
async fn save_sticker(
    pool: State<'_, sqlx::SqlitePool>,
    window: tauri::Window,
    markdown: &str,
) -> Result<(), String> {
    repository::update_sticker_markdown(&pool, window.label(), markdown)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn new_sticker(
    pool: State<'_, sqlx::SqlitePool>,
    handle: tauri::AppHandle,
) -> Result<(), String> {
    new_sticker_pool(&pool, handle).await
}

async fn new_sticker_pool(pool: &sqlx::SqlitePool, handle: tauri::AppHandle) -> Result<(), String> {
    let label = Uuid::new_v4();

    let w = tauri::WindowBuilder::new(&handle, label, tauri::WindowUrl::App("index.html".into()))
        .title("mdsticker")
        .hidden_title(true)
        .fullscreen(false)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .build()
        .map_err(|e| e.to_string())?;

    let pos = w.outer_position().map_err(|e| e.to_string())?;
    let size = w.outer_size().map_err(|e| e.to_string())?;

    let sticker = Sticker::new(w.label(), pos.x, pos.y, size.height, size.width);

    repository::insert_sticker(&pool, sticker)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_sticker(
    pool: State<'_, sqlx::SqlitePool>,
    window: tauri::Window,
) -> Result<(), String> {
    repository::remove_sticker(&pool, window.label())
        .await
        .map_err(|e| e.to_string())?;
    let _ = window.close();
    Ok(())
}

async fn restore_stickers(pool: &sqlx::SqlitePool, handle: tauri::AppHandle) -> Result<(), String> {
    let stickers = list_stickers(pool).await.unwrap();

    if stickers.len() == 0 {
        println!("no restore. create new sticker.");
        return new_sticker_pool(pool, handle).await;
    }

    println!("restore {} stickers", stickers.len());

    for sticker in stickers {
        let w = tauri::WindowBuilder::new(
            &handle,
            &sticker.uuid,
            tauri::WindowUrl::App("index.html".into()),
        )
        .title("mdsticker")
        .hidden_title(true)
        .fullscreen(false)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .position(sticker.pos_x.into(), sticker.pos_y.into())
        .inner_size(sticker.width.into(), sticker.height.into())
        .build()
        .map_err(|e| e.to_string())?;

        w.clone().once("init-request", move |_event| {
            let _ = w.emit("init-response", sticker);
        });
    }

    Ok(())
}

fn app_path(app: AppHandle) -> PathBuf {
    #[allow(deprecated)] // FIXME: Change to non-deprecated function in Tauri v2
    app.path_resolver()
        .app_dir()
        .expect("No App path was found!")
}

fn path_mapper(mut app_path: PathBuf, connection_string: &str) -> String {
    app_path.push(
        connection_string
            .split_once(':')
            .expect("Couldn't parse the connection string for DB!")
            .1,
    );

    format!(
        "sqlite:{}",
        app_path
            .to_str()
            .expect("Problem creating fully qualified path to Database file!")
    )
}

fn main() {
    let new_sticker_menu = CustomMenuItem::new("new_sticker".to_string(), "New sticker");
    let quit_menu = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(new_sticker_menu)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit_menu);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_sticker,
            new_sticker,
            remove_sticker,
            toggle_sticker_pinned
        ])
        .system_tray(system_tray)
        .setup(|app| {
            let dir = app_path(app.handle());
            std::fs::create_dir_all(&dir).expect("create app dir failed");
            let fqdb = path_mapper(dir, "sqlite:app.db");
            let exists = block_on(sqlx::sqlite::Sqlite::database_exists(&fqdb)).unwrap_or(false);
            let pool = block_on(repository::create_sqlite_pool(&fqdb))?;

            if !exists {
                block_on(repository::migrate_database(&pool))?;
            }

            block_on(restore_stickers(&pool, app.handle()))?;

            app.manage(pool);

            Ok(())
        })
        .on_window_event(|event| {
            let handle = event.window().app_handle();
            let pool = handle.state();

            match event.event() {
                WindowEvent::Resized(size) => {
                    println!(
                        "resized: {} width={} height={}",
                        event.window().label(),
                        size.width,
                        size.height
                    );
                    let _ = block_on(repository::update_sticker_size(
                        &pool,
                        event.window().label(),
                        size.width,
                        size.height,
                    ));
                }
                WindowEvent::Moved(position) => {
                    println!(
                        "moved: {} x={} y={}",
                        event.window().label(),
                        position.x,
                        position.y
                    );
                    let _ = block_on(repository::update_sticker_position(
                        &pool,
                        event.window().label(),
                        position.x,
                        position.y,
                    ));
                }
                _ => {}
            }
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "new_sticker" => {
                    let _ = block_on(new_sticker(app.state(), app.app_handle()));
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
