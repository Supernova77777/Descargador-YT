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
import { DownloaderService } from './downloader.service';
import {
  AudioQuality,
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

  // --- Exposed signals from service
  readonly videoInfo    = this.svc.videoInfo;
  readonly progress     = this.svc.progress;
  readonly isLoading    = this.svc.isLoading;
  readonly isDownloading = this.svc.isDownloading;
  readonly errorMsg     = this.svc.errorMsg;
  readonly lastRecord   = this.svc.lastRecord;
  readonly isIdle       = this.svc.isIdle;

  // --- Local component state
  url     = signal('');
  quality = signal<AudioQuality>('192');
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
      fetching_info: 'Obteniendo info…',
      downloading:   'Descargando audio…',
      converting:    'Convirtiendo a MP3…',
      complete:      '¡Descarga completa!',
      error:         'Error en la descarga',
    };
    return stage ? labels[stage] ?? stage : '';
  });

  readonly qualities: { value: AudioQuality; label: string }[] = [
    { value: '128', label: '128 kbps — Normal' },
    { value: '192', label: '192 kbps — Alta' },
    { value: '320', label: '320 kbps — Máxima ✨' },
  ];

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
      quality:    this.quality(),
    });
  }

  async onCancel(): Promise<void> {
    await this.svc.cancelDownload();
  }

  onPaste(event: ClipboardEvent): void {
    const text = event.clipboardData?.getData('text') ?? '';
    if (text.includes('youtube.com') || text.includes('youtu.be')) {
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
