// metadata.rs — Obtener información del video antes de descargar (Cobalt API / oEmbed)

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
struct OEmbedResponse {
    title: Option<String>,
    author_name: Option<String>,
    thumbnail_url: Option<String>,
}

/// Tauri command: obtiene la metadata del video usando YouTube oEmbed
#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    _get_video_info(&url)
        .await
        .map_err(|e| e.to_string())
}

async fn _get_video_info(url: &str) -> Result<VideoInfo> {
    let client = Client::new();
    let oembed_url = format!("https://www.youtube.com/oembed?url={}&format=json", url);

    let res = client.get(&oembed_url).send().await
        .map_err(|e| anyhow::anyhow!("Error conectando con YouTube: {}", e))?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Video no encontrado o privado."));
    }

    let info: OEmbedResponse = res.json().await
        .map_err(|e| anyhow::anyhow!("Error decodificando respuesta de YouTube: {}", e))?;

    let parts: Vec<&str> = url.split("v=").collect();
    let id = if parts.len() > 1 {
        parts[1].split('&').next().unwrap_or("unknown").to_string()
    } else {
        "unknown".to_string()
    };

    Ok(VideoInfo {
        id,
        title: info.title.unwrap_or_else(|| "Video Desconocido".into()),
        uploader: info.author_name.unwrap_or_else(|| "Autor Desconocido".into()),
        duration: 0, // No proveído por oEmbed
        thumbnail: info.thumbnail_url.unwrap_or_else(|| "https://via.placeholder.com/480x360.png?text=No+Thumb".into()),
        view_count: 0,
        upload_date: "N/A".to_string(),
    })
}
