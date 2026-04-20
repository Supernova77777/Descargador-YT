// metadata.rs — Obtener información del video antes de descargar

use crate::utils::platform;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::AppHandle;
use tokio::process::Command;

/// Struct que se devuelve al frontend (espejo del modelo TypeScript `VideoInfo`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub duration: u64,
    pub thumbnail: String,
    pub view_count: u64,
    pub upload_date: String,
}

/// Struct interna para deserializar el JSON de yt-dlp --dump-json
#[derive(Debug, Deserialize)]
struct YtDlpInfo {
    id: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
    duration: Option<u64>,
    thumbnail: Option<String>,
    view_count: Option<u64>,
    upload_date: Option<String>,
}

/// Tauri command: obtiene la metadata del video sin descargarlo.
#[tauri::command]
pub async fn get_video_info(
    app: AppHandle,
    url: String,
) -> Result<VideoInfo, String> {
    _get_video_info(&app, &url)
        .await
        .map_err(|e| e.to_string())
}

async fn _get_video_info(app: &AppHandle, url: &str) -> Result<VideoInfo> {
    let ytdlp = crate::utils::platform::get_ytdlp_path(app)?;

    let output = tokio::process::Command::new(&ytdlp)
        .args([
            "--dump-json",
            "--no-playlist",
            "--no-warnings",
            "--quiet",
            url,
        ])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Error ejecutando yt-dlp: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp error: {err}");
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    // yt-dlp puede emitir varias líneas JSON (playlists); tomamos la primera
    let first_line = raw.lines().next().unwrap_or_default();

    let info: YtDlpInfo = serde_json::from_str(first_line)
        .map_err(|e| anyhow::anyhow!("Error parseando JSON de yt-dlp: {e}"))?;

    Ok(VideoInfo {
        id:          info.id.unwrap_or_default(),
        title:       info.title.unwrap_or_else(|| "Sin título".into()),
        uploader:    info.uploader.unwrap_or_else(|| "Desconocido".into()),
        duration:    info.duration.unwrap_or(0),
        thumbnail:   info.thumbnail.unwrap_or_default(),
        view_count:  info.view_count.unwrap_or(0),
        upload_date: info.upload_date.unwrap_or_default(),
    })
}
