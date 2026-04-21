import {
  Component,
  OnInit,
  inject,
  signal,
  computed,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { DomSanitizer, SafeResourceUrl } from '@angular/platform-browser';
import { DownloaderService } from './downloader.service';
import { convertFileSrc } from '@tauri-apps/api/core';
import {
  formatDuration,
  formatViews,
} from '../../models/download.models';

@Component({
  selector: 'app-downloader',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './downloader.component.html',
  styleUrl: './downloader.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DownloaderComponent implements OnInit {
  private svc = inject(DownloaderService);
  private sanitizer = inject(DomSanitizer);

  readonly videoInfo    = this.svc.videoInfo;
  readonly progress     = this.svc.progress;
  readonly isLoading    = this.svc.isLoading;
  readonly isDownloading = this.svc.isDownloading;
  readonly errorMsg     = this.svc.errorMsg;
  readonly lastRecord   = this.svc.lastRecord;
  readonly isIdle       = this.svc.isIdle;

  url     = signal('');
  format  = signal<string>('mp3');
  quality = signal<string>('192');
  outputDir = signal('');

  readonly canFetch = computed(
    () => this.url().trim().length > 0 && this.isIdle()
  );

  readonly canDownload = computed(
    () =>
      !!this.videoInfo() &&
      this.outputDir().trim().length > 0 &&
      this.isIdle()
  );

  readonly progressPercent = computed(
    () => this.progress()?.percent ?? 0
  );

  readonly stageLabel = computed(() => {
    const stage = this.progress()?.stage;
    const labels: Record<string, string> = {
      fetching_info: 'Contactando Cobalt API…',
      downloading:   'Descargando Stream…',
      complete:      '¡Descarga completa!',
      error:         'Error en la descarga',
    };
    return stage ? labels[stage] ?? stage : '';
  });

  readonly currentQualities = computed(() => {
    if (this.format() === 'mp3') {
      return [
        { value: '128', label: '128 kbps — Normal' },
        { value: '192', label: '192 kbps — Alta' },
        { value: '320', label: '320 kbps — Máxima ✨' },
      ];
    } else {
      return [
        { value: '720', label: '720p — HD' },
        { value: '1080', label: '1080p — Full HD ✨' },
        { value: '1440', label: '1440p — 2K' },
        { value: '2160', label: '2160p — 4K' },
      ];
    }
  });

  readonly playerUrl = computed<SafeResourceUrl | null>(() => {
    const rec = this.lastRecord();
    if (!rec || !rec.output_path) return null;
    try {
      const url = convertFileSrc(rec.output_path);
      return this.sanitizer.bypassSecurityTrustResourceUrl(url);
    } catch {
      return null;
    }
  });

  formatDuration = formatDuration;
  formatViews    = formatViews;

  async ngOnInit(): Promise<void> {
    const dir = await this.svc.getDefaultOutputDir();
    this.outputDir.set(dir);
  }

  async onFetch(): Promise<void> {
    if (!this.canFetch()) return;
    await this.svc.fetchVideoInfo(this.url().trim());
  }

  async onPickDir(): Promise<void> {
    const dir = await this.svc.pickOutputDirectory();
    if (dir) this.outputDir.set(dir);
  }

  async onDownload(): Promise<void> {
    if (!this.canDownload()) return;
    await this.svc.startDownload({
      url:        this.url().trim(),
      output_dir: this.outputDir(),
      format:     this.format(),
      quality:    (this.format() === 'mp4' && this.quality() === '192') ? '720' : this.quality(),
    });
  }

  async onCancel(): Promise<void> {
    await this.svc.cancelDownload();
  }

  onPaste(event: ClipboardEvent): void {
    const text = event.clipboardData?.getData('text') ?? '';
    if (text.includes('youtube.com') || text.includes('youtu.be') || text.includes('tiktok.com') || text.includes('x.com')) {
      this.url.set(text.trim());
      setTimeout(() => this.onFetch(), 200);
    }
  }

  onClearUrl(): void {
    this.url.set('');
    this.svc.videoInfo.set(null);
    this.svc.errorMsg.set(null);
    this.svc.progress.set(null);
    this.svc.lastRecord.set(null);
  }
}
