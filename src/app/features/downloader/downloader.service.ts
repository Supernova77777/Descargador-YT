import { Injectable, signal, computed } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import {
  VideoInfo,
  DownloadParams,
  ProgressPayload,
  DownloadRecord,
  AppSettings,
  AudioQuality,
} from '../../models/download.models';

@Injectable({ providedIn: 'root' })
export class DownloaderService {
  // -------------------------------------------------------
  // Reactive State (Angular Signals)
  // -------------------------------------------------------
  readonly videoInfo    = signal<VideoInfo | null>(null);
  readonly progress     = signal<ProgressPayload | null>(null);
  readonly isLoading    = signal(false);
  readonly isDownloading = signal(false);
  readonly errorMsg     = signal<string | null>(null);
  readonly lastRecord   = signal<DownloadRecord | null>(null);

  readonly isIdle = computed(
    () => !this.isLoading() && !this.isDownloading()
  );

  private _unlistenProgress: UnlistenFn | null = null;

  // -------------------------------------------------------
  // Fetch video metadata
  // -------------------------------------------------------
  async fetchVideoInfo(url: string): Promise<void> {
    this.videoInfo.set(null);
    this.errorMsg.set(null);
    this.isLoading.set(true);

    try {
      const info = await invoke<VideoInfo>('get_video_info', { url });
      this.videoInfo.set(info);
    } catch (e: unknown) {
      this.errorMsg.set(this.parseError(e));
    } finally {
      this.isLoading.set(false);
    }
  }

  // -------------------------------------------------------
  // Pick output directory via native dialog
  // -------------------------------------------------------
  async pickOutputDirectory(): Promise<string | null> {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Selecciona carpeta de destino',
    });
    return selected as string | null;
  }

  // -------------------------------------------------------
  // Start download
  // -------------------------------------------------------
  async startDownload(params: DownloadParams): Promise<void> {
    this.errorMsg.set(null);
    this.progress.set(null);
    this.lastRecord.set(null);
    this.isDownloading.set(true);

    // Subscribe to progress events from Rust
    this._unlistenProgress = await listen<ProgressPayload>(
      'download://progress',
      (event) => this.progress.set(event.payload)
    );

    try {
      const record = await invoke<DownloadRecord>('download_audio', { params });
      this.lastRecord.set(record);
    } catch (e: unknown) {
      this.errorMsg.set(this.parseError(e));
    } finally {
      this.isDownloading.set(false);
      this._cleanup();
    }
  }

  // -------------------------------------------------------
  // Cancel download
  // -------------------------------------------------------
  async cancelDownload(): Promise<void> {
    try {
      await invoke('cancel_download');
    } finally {
      this.isDownloading.set(false);
      this.progress.set(null);
      this._cleanup();
    }
  }

  // -------------------------------------------------------
  // History
  // -------------------------------------------------------
  async getHistory(): Promise<DownloadRecord[]> {
    return invoke<DownloadRecord[]>('get_history');
  }

  async clearHistory(): Promise<void> {
    return invoke<void>('clear_history');
  }

  // -------------------------------------------------------
  // Settings
  // -------------------------------------------------------
  async loadSettings(): Promise<AppSettings> {
    return invoke<AppSettings>('get_settings');
  }

  async saveSettings(settings: AppSettings): Promise<void> {
    return invoke<void>('save_settings', { settings });
  }

  async getDefaultOutputDir(): Promise<string> {
    return invoke<string>('get_default_output_dir');
  }

  // -------------------------------------------------------
  // Helpers
  // -------------------------------------------------------
  private _cleanup(): void {
    if (this._unlistenProgress) {
      this._unlistenProgress();
      this._unlistenProgress = null;
    }
  }

  private parseError(e: unknown): string {
    if (typeof e === 'string') return e;
    if (e instanceof Error) return e.message;
    return 'Error desconocido. Intenta de nuevo.';
  }

  qualityLabel(q: AudioQuality): string {
    const labels: Record<AudioQuality, string> = {
      '128': '128 kbps — Normal',
      '192': '192 kbps — Alta',
      '320': '320 kbps — Máxima',
    };
    return labels[q];
  }
}
