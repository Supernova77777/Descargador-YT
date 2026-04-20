// download.rs — Lógica de descarga MP3 con progreso en tiempo real

use crate::utils::{platform, progress as prog};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
};
use uuid::Uuid;

// ──────────────────────────────────────────────────────────────
// Tipos públicos
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadParams {
    pub url:        String,
    pub output_dir: String,
    pub quality:    String, // "128" | "192" | "320"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id:           String,
    pub title:        String,
    pub uploader:     String,
    pub thumbnail:    String,
    pub duration:     u64,
    pub quality:      String,
    pub output_path:  String,
    pub downloaded_at: String,
    pub file_size_mb: f64,
}

// ──────────────────────────────────────────────────────────────
// Estado global de cancelación
// ──────────────────────────────────────────────────────────────

static CANCEL_FLAG: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

fn cancel_flag() -> &'static Arc<AtomicBool> {
    CANCEL_FLAG.get_or_init(|| Arc::new(AtomicBool::new(false)))
}

// ──────────────────────────────────────────────────────────────
// Comandos Tauri
// ──────────────────────────────────────────────────────────────

/// Descarga audio de YouTube y lo convierte a MP3.
/// Emite eventos `download://progress` mientras descarga.
#[tauri::command]
pub async fn download_audio(
    app: AppHandle,
    params: DownloadParams,
) -> Result<DownloadRecord, String> {
    cancel_flag().store(false, Ordering::SeqCst);
    _download_audio(&app, params).await.map_err(|e| e.to_string())
}

/// Cancela la descarga en curso.
#[tauri::command]
pub async fn cancel_download() -> Result<(), String> {
    cancel_flag().store(true, Ordering::SeqCst);
    tracing::info!("Descarga cancelada por el usuario");
    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Lógica interna
// ──────────────────────────────────────────────────────────────

async fn _download_audio(
    app: &AppHandle,
    params: DownloadParams,
) -> anyhow::Result<DownloadRecord> {
    let ytdlp   = platform::get_ytdlp_path(app)?;
    let ffmpeg  = platform::get_ffmpeg_path(app)?;
    let out_dir = PathBuf::from(&params.output_dir);

    anyhow::ensure!(out_dir.exists(), "La carpeta de destino no existe: {}", out_dir.display());

    // Emitir evento inicial "fetching_info"
    emit_progress(app, prog::ProgressPayload {
        percent: 0.0,
        speed:   String::new(),
        eta:     String::new(),
        stage:   prog::DownloadStage::FetchingInfo,
    });

    // Primero obtenemos metadata para el registro del historial
    let info = crate::commands::metadata::get_video_info(
        app.clone(),
        params.url.clone(),
    ).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    // Plantilla de salida: <titulo>.%(ext)s
    let output_template = out_dir
        .join("%(title)s.%(ext)s")
        .to_string_lossy()
        .into_owned();

    // Calidad → bitrate para ffmpeg
    let audio_quality = match params.quality.as_str() {
        "128" => "128K",
        "320" => "320K",
        _     => "192K",   // default 192
    };

    // Construir comando yt-dlp
    let mut cmd = Command::new(&ytdlp);
    cmd.args([
        "--no-playlist",
        "--extract-audio",
        "--audio-format", "mp3",
        "--audio-quality", audio_quality,
        "--ffmpeg-location", &ffmpeg.to_string_lossy(),
        "--newline",           // progreso línea a línea
        "--progress",
        "--no-warnings",
        "-o", &output_template,
        &params.url,
    ])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    // Leer stderr línea a línea para progreso
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::debug!("yt-dlp: {line}");
                if let Some(payload) = prog::parse_ytdlp_line(&line) {
                    emit_progress(&app_clone, payload);
                }
                // Revisamos cancelación
                if cancel_flag().load(Ordering::SeqCst) {
                    break;
                }
            }
        });
    }

    // También leer stdout (yt-dlp a veces escribe progreso en stdout)
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(payload) = prog::parse_ytdlp_line(&line) {
                    emit_progress(&app_clone, payload);
                }
            }
        });
    }

    // Esperar a que termine el proceso (con chequeo de cancelación)
    loop {
        if cancel_flag().load(Ordering::SeqCst) {
            let _ = child.kill().await;
            anyhow::bail!("Descarga cancelada por el usuario");
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    anyhow::bail!("yt-dlp terminó con error. Código: {:?}", status.code());
                }
                break;
            }
            Ok(None) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
            Err(e) => {
                anyhow::bail!("Error esperando a yt-dlp: {e}");
            }
        }
    }

    // Encontrar el archivo MP3 generado
    let mp3_path = find_mp3_file(&out_dir, &info.title)?;
    let file_size_mb = std::fs::metadata(&mp3_path)
        .map(|m| m.len() as f64 / 1_048_576.0)
        .unwrap_or(0.0);

    // Evento de completado
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
        output_path:  mp3_path.to_string_lossy().into_owned(),
        downloaded_at: Utc::now().to_rfc3339(),
        file_size_mb,
    };

    // Persistir en historial
    crate::commands::settings::append_history(app, record.clone()).await?;

    tracing::info!("✅ Descarga completa: {}", record.output_path);
    Ok(record)
}

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

fn emit_progress(app: &AppHandle, payload: prog::ProgressPayload) {
    let _ = app.emit("download://progress", &payload);
}

/// Busca el archivo MP3 más reciente en el directorio de salida que contenga
/// parte del título (tolerante a caracteres especiales que yt-dlp sanitiza).
fn find_mp3_file(dir: &PathBuf, _title: &str) -> anyhow::Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("mp3"))
                .unwrap_or(false)
        })
        .collect();

    if candidates.is_empty() {
        anyhow::bail!("No se encontró ningún archivo MP3 en {}", dir.display());
    }

    // Ordenar por tiempo de modificación (más reciente primero)
    candidates.sort_by(|a, b| {
        let ta = a.metadata().and_then(|m| m.modified()).ok();
        let tb = b.metadata().and_then(|m| m.modified()).ok();
        tb.cmp(&ta)
    });

    Ok(candidates.remove(0))
}
