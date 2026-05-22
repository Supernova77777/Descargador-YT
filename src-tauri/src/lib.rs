// lib.rs — Tauri app setup y registro de comandos

mod commands;
mod utils;

use commands::{download, metadata, settings as cfg};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializar tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tauri::Builder::default()
        // ── Plugins ──
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        // ── Setup ──
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Aseguramos que el archivo de historial existe
            tauri::async_runtime::spawn(async move {
                if let Err(e) = cfg::ensure_app_dir(&app_handle).await {
                    tracing::error!("Error inicializando directorio de la app: {e}");
                }
            });

            #[cfg(desktop)]
            let _ = app.deep_link().register_all();

            if let Ok(Some(urls)) = app.deep_link().get_current() {
                if let Some(url) = urls.first() {
                    let mut init_url = commands::download::INITIAL_URL.lock().unwrap();
                    *init_url = Some(url.to_string());
                }
            }

            let app_handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                if let Some(url) = event.urls().first() {
                    let _ = app_handle.emit("shared-url", url.to_string());
                }
            });

            Ok(())
        })
        // ── Comandos ──
        .invoke_handler(tauri::generate_handler![
            metadata::get_video_info,
            download::download_audio,
            download::cancel_download,
            cfg::get_settings,
            cfg::save_settings,
            cfg::get_default_output_dir,
            cfg::get_history,
            cfg::clear_history,
            download::get_initial_shared_url,
        ])
        .run(tauri::generate_context!())
        .expect("Error al iniciar la aplicación Tauri");
}
