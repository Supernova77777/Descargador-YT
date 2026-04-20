// settings.rs — Configuración de la app e historial de descargas

use crate::commands::download::DownloadRecord;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "settings.json";
const HISTORY_FILE:  &str = "history.json";

// ──────────────────────────────────────────────────────────────
// Tipos
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_output_dir:       String,
    pub default_quality:          String,
    pub max_concurrent_downloads: u8,
    pub auto_open_folder:         bool,
    pub theme:                    String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_output_dir:       default_music_dir(),
            default_quality:          "192".into(),
            max_concurrent_downloads: 1,
            auto_open_folder:         true,
            theme:                    "dark".into(),
        }
    }
}

fn default_music_dir() -> String {
    dirs::audio_dir()
        .or_else(dirs::download_dir)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .to_string_lossy()
        .into_owned()
}

// ──────────────────────────────────────────────────────────────
// Inicialización
// ──────────────────────────────────────────────────────────────

pub async fn ensure_app_dir(app: &AppHandle) -> Result<()> {
    let dir = app_data_dir(app)?;
    if !dir.exists() {
        tokio::fs::create_dir_all(&dir).await?;
    }
    Ok(())
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("No se pudo obtener app_data_dir: {e}"))
}

// ──────────────────────────────────────────────────────────────
// Comandos Tauri
// ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    _get_settings(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    settings: AppSettings,
) -> Result<(), String> {
    _save_settings(&app, &settings).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_default_output_dir(app: AppHandle) -> Result<String, String> {
    let settings = _get_settings(&app).await.map_err(|e| e.to_string())?;
    Ok(settings.default_output_dir)
}

#[tauri::command]
pub async fn get_history(app: AppHandle) -> Result<Vec<DownloadRecord>, String> {
    _get_history(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_history(app: AppHandle) -> Result<(), String> {
    let dir = app_data_dir(&app).map_err(|e| e.to_string())?;
    let path = dir.join(HISTORY_FILE);
    if path.exists() {
        tokio::fs::remove_file(&path).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Funciones internas
// ──────────────────────────────────────────────────────────────

async fn _get_settings(app: &AppHandle) -> Result<AppSettings> {
    let path = app_data_dir(app)?.join(SETTINGS_FILE);

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let text = tokio::fs::read_to_string(&path).await?;
    let settings: AppSettings = serde_json::from_str(&text)
        .unwrap_or_else(|_| AppSettings::default());

    Ok(settings)
}

async fn _save_settings(app: &AppHandle, settings: &AppSettings) -> Result<()> {
    let dir = app_data_dir(app)?;
    tokio::fs::create_dir_all(&dir).await?;

    let path = dir.join(SETTINGS_FILE);
    let text = serde_json::to_string_pretty(settings)?;
    tokio::fs::write(path, text).await?;

    Ok(())
}

async fn _get_history(app: &AppHandle) -> Result<Vec<DownloadRecord>> {
    let path = app_data_dir(app)?.join(HISTORY_FILE);

    if !path.exists() {
        return Ok(vec![]);
    }

    let text = tokio::fs::read_to_string(&path).await?;
    let records: Vec<DownloadRecord> = serde_json::from_str(&text)
        .unwrap_or_default();

    Ok(records)
}

pub async fn append_history(app: &AppHandle, record: DownloadRecord) -> Result<()> {
    let mut records = _get_history(app).await.unwrap_or_default();
    records.insert(0, record);       // más reciente primero
    records.truncate(100);           // máximo 100 entradas

    let dir = app_data_dir(app)?;
    tokio::fs::create_dir_all(&dir).await?;

    let path = dir.join(HISTORY_FILE);
    let text = serde_json::to_string_pretty(&records)?;
    tokio::fs::write(path, text).await?;

    Ok(())
}
