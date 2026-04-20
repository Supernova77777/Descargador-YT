// platform.rs — Resolución de binarios cross-platform

use anyhow::{Context, Result};
use std::path::PathBuf;
use tauri::AppHandle;

pub fn get_ytdlp_path(app: &AppHandle) -> Result<PathBuf> {
    use tauri::Manager;

    let resource_dir = app
        .path()
        .resource_dir()
        .context("No se pudo obtener resource_dir")?;

    #[cfg(target_os = "windows")]
    let (binary_full, binary_short) = ("yt-dlp-x86_64-pc-windows-msvc.exe", "yt-dlp.exe");

    #[cfg(target_os = "macos")]
    let (binary_full, binary_short) = ("yt-dlp-x86_64-apple-darwin", "yt-dlp");

    #[cfg(target_os = "linux")]
    let (binary_full, binary_short) = ("yt-dlp-x86_64-unknown-linux-gnu", "yt-dlp");

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    let (binary_full, binary_short) = ("yt-dlp", "yt-dlp");

    let candidates = vec![
        resource_dir.join("binaries").join(binary_full),
        resource_dir.join(binary_full),
        resource_dir.join(binary_short),
        resource_dir.join("_up_").join("binaries").join(binary_full),
    ];

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!(
        "No se encontró el binario yt-dlp.\nDescárgalo y colócalo en src-tauri/binaries/. Buscado en: {:?}",
        resource_dir
    );
}

/// Retorna la ruta al binario `ffmpeg` correcto para la plataforma actual.
pub fn get_ffmpeg_path(app: &AppHandle) -> Result<PathBuf> {
    use tauri::Manager;

    let resource_dir = app
        .path()
        .resource_dir()
        .context("No se pudo obtener resource_dir")?;

    #[cfg(target_os = "windows")]
    let (binary_full, binary_short) = ("ffmpeg-x86_64-pc-windows-msvc.exe", "ffmpeg.exe");

    #[cfg(target_os = "macos")]
    let (binary_full, binary_short) = ("ffmpeg-x86_64-apple-darwin", "ffmpeg");

    #[cfg(target_os = "linux")]
    let (binary_full, binary_short) = ("ffmpeg-x86_64-unknown-linux-gnu", "ffmpeg");

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    let (binary_full, binary_short) = ("ffmpeg", "ffmpeg");

    let candidates = vec![
        resource_dir.join("binaries").join(binary_full),
        resource_dir.join(binary_full),
        resource_dir.join(binary_short),
        resource_dir.join("_up_").join("binaries").join(binary_full),
    ];

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!(
        "No se encontró el binario ffmpeg.\nDescárgalo y colócalo en src-tauri/binaries/. Buscado en: {:?}",
        resource_dir
    );
}
