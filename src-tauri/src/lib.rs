// lib.rs — Tauri app setup y registro de comandos

mod commands;
mod utils;

use commands::{download, metadata, settings as cfg};


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
        // ── Setup ──
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Aseguramos que el archivo de historial existe
            tauri::async_runtime::spawn(async move {
                if let Err(e) = cfg::ensure_app_dir(&app_handle).await {
                    tracing::error!("Error inicializando directorio de la app: {e}");
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
        ])
        .run(tauri::generate_context!())
        .expect("Error al iniciar la aplicación Tauri");
}
