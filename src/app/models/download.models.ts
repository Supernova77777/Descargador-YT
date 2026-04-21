// ====================================================
// Shared TypeScript models — mirroring Rust structs
// ====================================================

/** Info del video obtenida antes de descargar */
export interface VideoInfo {
  id: string;
  title: string;
  uploader: string;
  duration: number;          // segundos
  thumbnail: string;         // URL HTTPS
  view_count: number;
  upload_date: string;
}

/** Parámetros enviados al comando Rust `download_audio` */
export interface DownloadParams {
  url: string;
  output_dir: string;
  quality: string;
  format: string;
}

/** Calidades de audio disponibles */
export type AudioQuality = '128' | '192' | '320';

/** Payload de eventos de progreso emitidos desde Rust */
export interface ProgressPayload {
  percent: number;           // 0 – 100
  speed: string;             // e.g. "1.20 MiB/s"
  eta: string;               // e.g. "00:12"
  stage: DownloadStage;
}

export type DownloadStage =
  | 'fetching_info'
  | 'downloading'
  | 'converting'
  | 'complete'
  | 'error';

/** Registro guardado en historial */
export interface DownloadRecord {
  id: string;
  title: string;
  uploader: string;
  thumbnail: string;
  duration: number;
  quality: string;
  format: string;
  output_path: string;
  downloaded_at: string;     // ISO timestamp
  file_size_mb: number;
}

/** Configuración de la aplicación */
export interface AppSettings {
  default_output_dir: string;
  default_quality: AudioQuality;
  max_concurrent_downloads: number;
  auto_open_folder: boolean;
  theme: 'dark' | 'light';
}

/** Helper: formatea segundos en m:ss */
export function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

/** Helper: formatea número grande */
export function formatViews(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000)     return `${(n / 1_000).toFixed(0)}K`;
  return n.toString();
}
