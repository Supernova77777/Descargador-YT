// download.rs — Lógica de descarga nativa usando Cobalt API y reqwest

use crate::utils::progress as prog;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use reqwest::Client;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tokio::{fs::File, io::AsyncWriteExt};
use futures_util::StreamExt;
use uuid::Uuid;

// =================================================================================
// Tipos públicos
// =================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadParams {
    pub url:        String,
    pub output_dir: String,
    pub format:     String, // "mp3" o "mp4"
    pub quality:    String, // "128" | "192" | "320" (o resoluciones como "1080" si es video)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id:           String,
    pub title:        String,
    pub uploader:     String,
    pub thumbnail:    String,
    pub duration:     u64,
    #[serde(default = "default_quality")]
    pub quality:      String,
    #[serde(default = "default_format")]
    pub format:       String,
    pub output_path:  String,
    pub downloaded_at: String,
    pub file_size_mb: f64,
}

fn default_quality() -> String { "192".into() }
fn default_format() -> String { "mp3".into() }

#[derive(Deserialize)]
struct CobaltResponse {
    status: String,
    url: Option<String>,
    text: Option<String>,
}

// =================================================================================
// Estado global de cancelación
// =================================================================================

static CANCEL_FLAG: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

fn cancel_flag() -> &'static Arc<AtomicBool> {
    CANCEL_FLAG.get_or_init(|| Arc::new(AtomicBool::new(false)))
}

// =================================================================================
// Comandos Tauri
// =================================================================================

/// Descarga multimedia de YouTube usando Cobalt y graba el stream.
#[tauri::command]
pub async fn download_audio(
    app: AppHandle,
    params: DownloadParams,
) -> Result<DownloadRecord, String> {
    cancel_flag().store(false, Ordering::SeqCst);
    _download_media(&app, params).await.map_err(|e| e.to_string())
}

/// Cancela la descarga en curso.
#[tauri::command]
pub async fn cancel_download() -> Result<(), String> {
    cancel_flag().store(true, Ordering::SeqCst);
    tracing::info!("Descarga cancelada por el usuario");
    Ok(())
}

// =================================================================================
// Lógica interna
// =================================================================================

async fn _download_media(
    app: &AppHandle,
    params: DownloadParams,
) -> anyhow::Result<DownloadRecord> {
    let out_dir = PathBuf::from(&params.output_dir);
    anyhow::ensure!(out_dir.exists(), "La carpeta de destino no existe: {}", out_dir.display());

    emit_progress(app, prog::ProgressPayload {
        percent: 5.0,
        speed:   String::new(),
        eta:     String::new(),
        stage:   prog::DownloadStage::FetchingInfo,
    });

    let info = crate::commands::metadata::get_video_info(params.url.clone())
        .await.map_err(|e| anyhow::anyhow!("{e}"))?;

    let is_audio = params.format == "mp3";
    let cobalt_url = "https://cobalt.meowing.de/api/json"; // API Pública configurada sin JWT
    
    // Configurar el payload para audio (mp3) o video (mp4)
    let payload = if is_audio {
        json!({
            "url": params.url,
            "isAudioOnly": true,
            "aFormat": "mp3",
            "filenamePattern": "classic"
        })
    } else {
        json!({
            "url": params.url,
            "vQuality": "1080",
            "filenamePattern": "classic"
        })
    };

    let client = Client::new();
    let api_res = client.post(cobalt_url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Error contactando API Cobalt: {}", e))?;

    let cob_resp: CobaltResponse = api_res.json().await?;

    if cob_resp.status == "error" || cob_resp.status == "rate-limit" {
        return Err(anyhow::anyhow!("Cobalt API Error: {}", cob_resp.text.unwrap_or_else(|| "Error desconocido".into())));
    }

    let download_url = cob_resp.url.ok_or_else(|| anyhow::anyhow!("El servidor no devolvió una URL de stream válida"))?;

    emit_progress(app, prog::ProgressPayload {
        percent: 15.0,
        speed:   String::new(),
        eta:     String::new(),
        stage:   prog::DownloadStage::Downloading,
    });

    let stream_res = client.get(&download_url).send().await
        .map_err(|e| anyhow::anyhow!("Error descargando stream: {}", e))?;

    // Generar nombre de destino limpio
    let safe_title = info.title.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_");
    let ext = if is_audio { "mp3" } else { "mp4" };
    let filename = format!("{} ({}).{}", safe_title.trim(), Uuid::new_v4().to_string().chars().take(4).collect::<String>(), ext);
    let out_path = out_dir.join(&filename);

    let mut file = File::create(&out_path).await?;
    let total_size = stream_res.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    
    let mut stream = stream_res.bytes_stream();
    let start_time = std::time::Instant::now();

    while let Some(chunk_res) = stream.next().await {
        if cancel_flag().load(Ordering::SeqCst) {
            drop(file);
            let _ = tokio::fs::remove_file(&out_path).await;
            anyhow::bail!("Descarga cancelada por el usuario");
        }

        let chunk = chunk_res?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 && downloaded % (1024 * 512) == 0 { // Emit progress every ~512KB
            let pct = (downloaded as f64 / total_size as f64) * 85.0 + 15.0; 
            
            // Calc speed
            let elapsed = start_time.elapsed().as_secs_f64();
            let mbps = if elapsed > 0.0 { (downloaded as f64 / 1_048_576.0) / elapsed } else { 0.0 };
            
            let speed_lbl = format!("{:.1} MB/s", mbps);
            
            emit_progress(app, prog::ProgressPayload {
                percent: pct,
                speed:   speed_lbl,
                eta:     String::from("Calculando..."),
                stage:   prog::DownloadStage::Downloading,
            });
        }
    }

    let file_size_mb = downloaded as f64 / 1_048_576.0;

    emit_progress(app, prog::ProgressPayload {
        percent: 100.0,
        speed:   String::new(),
        eta:     String::from("00:00"),
        stage:   prog::DownloadStage::Complete,
    });

    let record = DownloadRecord {
        id:           Uuid::new_v4().to_string(),
        title:        info.title.clone(),
        uploader:     info.uploader.clone(),
        thumbnail:    info.thumbnail.clone(),
        duration:     info.duration,
        quality:      params.quality.clone(),
        format:       params.format.clone(),
        output_path:  out_path.to_string_lossy().into_owned(),
        downloaded_at: Utc::now().to_rfc3339(),
        file_size_mb,
    };

    crate::commands::settings::append_history(app, record.clone()).await?;
    tracing::info!("Media descargada nativamente: {}", record.output_path);
    Ok(record)
}

fn emit_progress(app: &AppHandle, payload: prog::ProgressPayload) {
    let _ = app.emit("download://progress", &payload);
}
