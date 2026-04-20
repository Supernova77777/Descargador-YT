// main.rs — Entry point (non-blocking)
// Tauri requiere que main.rs sea mínimo; toda la lógica va en lib.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    yt_mp3_downloader_lib::run();
}
