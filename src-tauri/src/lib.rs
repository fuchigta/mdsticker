// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod repository;

use rand::RngExt;
use std::path::PathBuf;

use repository::list_stickers;
use serde::{Deserialize, Serialize};
use tauri::{
    async_runtime::block_on, image::Image, menu::{Menu, MenuItem}, tray::TrayIconBuilder, AppHandle, Emitter, Manager, State, WindowEvent
};
use tauri_plugin_shell::ShellExt;
use uuid::Uuid;

fn generate_random_color() -> String {
    let mut rng = rand::rng(); // probe: temporary, to be reverted

    // 0-255の範囲で3つの数値（R,G,B）を生成
    let r: u8 = rng.random();
    let g: u8 = rng.random();
    let b: u8 = rng.random();

    // 16進数の文字列に変換（桁数を揃えるためにformat!マクロを使用）
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Sticker {
    uuid: String,
    markdown: String,
    color: String,
    pos_x: i32,
    pos_y: i32,
    height: u32,
    width: u32,
    pinned: bool,
    created_at: String,
    updated_at: String,
}

impl Sticker {
    pub fn new(uuid: &str, pos_x: i32, pos_y: i32, height: u32, width: u32) -> Self {
        Sticker {
            uuid: uuid.to_string(),
            markdown: "".to_string(),
            color: generate_random_color(),
            pos_x,
            pos_y,
            height,
            width,
            pinned: false,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }
    }
}

#[tauri::command]
async fn open_url(handle: tauri::AppHandle, url: &str) -> Result<(), String> {
    handle
        .app_handle()
        .shell()
        .open(url, None)
        .map_err(|e| e.to_string())?;
    Ok(())
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
async fn save_sticker_markdown(
    pool: State<'_, sqlx::SqlitePool>,
    window: tauri::Window,
    markdown: &str,
) -> Result<(), String> {
    repository::update_sticker_markdown(&pool, window.label(), markdown)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_sticker_color(
    pool: State<'_, sqlx::SqlitePool>,
    window: tauri::Window,
    color: &str,
) -> Result<(), String> {
    println!("color: {}", &color);
    repository::update_sticker_color(&pool, window.label(), color)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn new_sticker(
    pool: State<'_, sqlx::SqlitePool>,
    handle: tauri::AppHandle,
) -> Result<(), String> {
    new_sticker_pool(&pool, &handle).await
}

async fn new_sticker_pool(
    pool: &sqlx::SqlitePool,
    handle: &tauri::AppHandle,
) -> Result<(), String> {
    let label = Uuid::new_v4();

    let w = tauri::WebviewWindowBuilder::new(
        handle,
        label,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("")
    .fullscreen(false)
    .minimizable(false)
    .maximizable(false)
    .closable(false)
    .inner_size(500.0, 400.0)
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
    handle: tauri::AppHandle,
) -> Result<(), String> {
    repository::remove_sticker(&pool, window.label())
        .await
        .map_err(|e| e.to_string())?;
    let _ = window.close();

    match handle.webview_windows().get("trashbox") {
        Some(w) => {
            w.emit("reload", "").map_err(|e| e.to_string())?;
        }
        None => {}
    }

    Ok(())
}

#[tauri::command]
async fn load_sticker(
    pool: State<'_, sqlx::SqlitePool>,
    window: tauri::Window,
) -> Result<Sticker, String> {
    let sticker = repository::get_sticker(&pool, window.label())
        .await
        .map_err(|e| e.to_string())?;
    Ok(sticker)
}

#[tauri::command]
async fn load_trashbox_stickers(pool: State<'_, sqlx::SqlitePool>) -> Result<Vec<Sticker>, String> {
    let stickers = repository::list_archived_stickers(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(stickers)
}

#[tauri::command]
async fn delete_stickers(
    pool: State<'_, sqlx::SqlitePool>,
    stickers: Vec<Sticker>,
) -> Result<(), String> {
    repository::delete_stickers(&pool, &stickers)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn recover_stickers(
    pool: State<'_, sqlx::SqlitePool>,
    handle: tauri::AppHandle,
    stickers: Vec<Sticker>,
) -> Result<(), String> {
    repository::recover_stickers(&pool, &stickers)
        .await
        .map_err(|e| e.to_string())?;

    for sticker in &stickers {
        let _ = tauri::WebviewWindowBuilder::new(
            &handle,
            &sticker.uuid,
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("")
        .fullscreen(false)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .position(sticker.pos_x.into(), sticker.pos_y.into())
        .inner_size(sticker.width.into(), sticker.height.into())
        .always_on_top(sticker.pinned)
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn restore_stickers(
    pool: &sqlx::SqlitePool,
    handle: &tauri::AppHandle,
) -> Result<(), String> {
    let stickers = list_stickers(pool).await.unwrap();

    if stickers.len() == 0 {
        println!("no restore. create new sticker.");
        return new_sticker_pool(pool, handle).await;
    }

    println!("restore {} stickers", stickers.len());

    for sticker in stickers {
        let _ = tauri::WebviewWindowBuilder::new(
            handle,
            &sticker.uuid,
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("")
        .fullscreen(false)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .position(sticker.pos_x.into(), sticker.pos_y.into())
        .inner_size(sticker.width.into(), sticker.height.into())
        .always_on_top(sticker.pinned)
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn app_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("No App path was found!")
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            open_url,
            save_sticker_markdown,
            save_sticker_color,
            load_sticker,
            load_trashbox_stickers,
            delete_stickers,
            recover_stickers,
            new_sticker,
            remove_sticker,
            toggle_sticker_pinned
        ])
        .setup(|app| {
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "new_sticker", "New sticker", true, None::<&str>)?,
                    &MenuItem::with_id(app, "trashbox", "Trashbox", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                ],
            )?;
            let _tray = TrayIconBuilder::new()
                .icon_as_template(false)
                .icon(Image::from_bytes(include_bytes!("../icons/icon.png"))?)
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "new_sticker" => {
                        let _ = block_on(new_sticker_pool(&app.state(), app.app_handle()));
                    }
                    "trashbox" => {
                        let _ = tauri::WebviewWindowBuilder::new(
                            app.app_handle(),
                            "trashbox",
                            tauri::WebviewUrl::App("trashbox.html".into()),
                        )
                        .title("")
                        .build();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            let dir = app_path(app.handle());
            std::fs::create_dir_all(&dir).expect("create app dir failed");
            let pool = block_on(repository::create_sqlite_pool(&path_mapper(
                dir,
                "sqlite:app.db",
            )))?;

            block_on(repository::migrate_database(&pool))?;
            block_on(restore_stickers(&pool, app.handle()))?;

            app.manage(pool);

            Ok(())
        })
        .on_window_event(|window, event| {
            let handle = window.app_handle();
            let pool = handle.state();

            match event {
                WindowEvent::Resized(size) => {
                    println!(
                        "resized: {} width={} height={}",
                        window.label(),
                        size.width,
                        size.height
                    );
                    let _ = block_on(repository::update_sticker_size(
                        &pool,
                        window.label(),
                        size.width,
                        size.height,
                    ));
                }
                WindowEvent::Moved(position) => {
                    println!(
                        "moved: {} x={} y={}",
                        window.label(),
                        position.x,
                        position.y
                    );
                    let _ = block_on(repository::update_sticker_position(
                        &pool,
                        window.label(),
                        position.x,
                        position.y,
                    ));
                }
                _ => {}
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Number of cases per property. Reduced from proptest's default of 256
    /// to keep CI fast. Set in code (not via `PROPTEST_CASES`) so local runs
    /// and CI use the same value.
    const CASES: u32 = 64;

    /// `#rrggbb` with lowercase hex digits, as produced by `generate_random_color`.
    fn is_lower_hex_color(s: &str) -> bool {
        s.len() == 7
            && s.starts_with('#')
            && s[1..].chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
    }

    /// All fields of a sticker, for equality checks (`Sticker` does not derive
    /// `PartialEq`, and production code is left untouched).
    #[allow(clippy::type_complexity)]
    fn fields(s: &Sticker) -> (&str, &str, &str, i32, i32, u32, u32, bool, &str, &str) {
        (
            &s.uuid,
            &s.markdown,
            &s.color,
            s.pos_x,
            s.pos_y,
            s.height,
            s.width,
            s.pinned,
            &s.created_at,
            &s.updated_at,
        )
    }

    fn arb_sticker() -> impl Strategy<Value = Sticker> {
        (
            any::<String>(),
            any::<String>(),
            any::<String>(),
            any::<i32>(),
            any::<i32>(),
            any::<u32>(),
            any::<u32>(),
            any::<bool>(),
            any::<String>(),
            any::<String>(),
        )
            .prop_map(
                |(
                    uuid,
                    markdown,
                    color,
                    pos_x,
                    pos_y,
                    height,
                    width,
                    pinned,
                    created_at,
                    updated_at,
                )| {
                    Sticker {
                        uuid,
                        markdown,
                        color,
                        pos_x,
                        pos_y,
                        height,
                        width,
                        pinned,
                        created_at,
                        updated_at,
                    }
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(CASES))]

        /// The connection string is always split at its first `:`. The scheme
        /// part is generated without `:` and the rest may contain any `:`, so
        /// every input contains at least one `:` (inputs without one make
        /// `path_mapper` panic by design and are out of scope here).
        #[test]
        fn path_mapper_has_sqlite_prefix_and_db_suffix(
            app_dir in "[a-zA-Z0-9_./-]{0,32}",
            scheme in "[^:]{0,8}",
            db in any::<String>(),
        ) {
            let mapped = path_mapper(PathBuf::from(app_dir), &format!("{scheme}:{db}"));
            prop_assert!(mapped.starts_with("sqlite:"), "{mapped:?}");
            prop_assert!(mapped.ends_with(&db), "{mapped:?} does not end with {db:?}");
        }

        #[test]
        fn random_color_is_lowercase_hex(calls in 1usize..16) {
            for _ in 0..calls {
                let color = generate_random_color();
                prop_assert!(is_lower_hex_color(&color), "{color:?}");
            }
        }

        #[test]
        fn new_sticker_has_initial_values(
            uuid in any::<String>(),
            pos_x in any::<i32>(),
            pos_y in any::<i32>(),
            height in any::<u32>(),
            width in any::<u32>(),
        ) {
            let s = Sticker::new(&uuid, pos_x, pos_y, height, width);
            prop_assert_eq!(&s.uuid, &uuid);
            prop_assert_eq!((s.pos_x, s.pos_y, s.height, s.width), (pos_x, pos_y, height, width));
            prop_assert_eq!(&s.markdown, "");
            prop_assert!(!s.pinned);
            prop_assert!(is_lower_hex_color(&s.color), "{:?}", s.color);
        }

        #[test]
        fn sticker_json_round_trip(sticker in arb_sticker()) {
            let json = serde_json::to_string(&sticker).unwrap();
            let decoded: Sticker = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(fields(&decoded), fields(&sticker));
        }
    }
}
