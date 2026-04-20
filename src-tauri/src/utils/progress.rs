// progress.rs — Parsing de la salida de yt-dlp para extraer progreso

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStage {
    FetchingInfo,
    Downloading,
    Converting,
    Complete,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPayload {
    pub percent: f64,
    pub speed: String,
    pub eta: String,
    pub stage: DownloadStage,
}

impl Default for ProgressPayload {
    fn default() -> Self {
        Self {
            percent: 0.0,
            speed: String::new(),
            eta: String::new(),
            stage: DownloadStage::Downloading,
        }
    }
}

/// Parsea una línea de stderr/stdout de yt-dlp y retorna un `ProgressPayload` si aplica.
///
/// Ejemplo de línea de yt-dlp:
/// `[download]  34.5% of   5.23MiB at   1.20MiB/s ETA 00:03`
pub fn parse_ytdlp_line(line: &str) -> Option<ProgressPayload> {
    // Línea de progreso de descarga
    if line.contains("[download]") && line.contains('%') {
        let re = Regex::new(
            r"\[download\]\s+(\d+(?:\.\d+)?)%.*?(?:at\s+([\d.]+\s*\S+))?\s*(?:ETA\s+([\d:]+))?",
        )
        .ok()?;

        if let Some(caps) = re.captures(line) {
            let percent: f64 = caps
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0.0);
            let speed = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let eta = caps
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            return Some(ProgressPayload {
                percent,
                speed,
                eta,
                stage: DownloadStage::Downloading,
            });
        }
    }

    // Conversión a MP3 (ffmpeg post-processing)
    if line.contains("[ExtractAudio]") || line.contains("Destination:") {
        return Some(ProgressPayload {
            percent: 99.0,
            speed: String::new(),
            eta: String::new(),
            stage: DownloadStage::Converting,
        });
    }

    // Completado
    if line.contains("[download] 100%") {
        return Some(ProgressPayload {
            percent: 100.0,
            speed: String::new(),
            eta: String::from("00:00"),
            stage: DownloadStage::Downloading,
        });
    }

    None
}
