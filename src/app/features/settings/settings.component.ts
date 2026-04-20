import {
  Component,
  OnInit,
  inject,
  signal,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { DownloaderService } from '../downloader/downloader.service';
import { AppSettings, AudioQuality } from '../../models/download.models';

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsComponent implements OnInit {
  private svc = inject(DownloaderService);

  settings = signal<AppSettings>({
    default_output_dir: '',
    default_quality: '192',
    max_concurrent_downloads: 1,
    auto_open_folder: true,
    theme: 'dark',
  });

  isSaving = signal(false);
  saved    = signal(false);

  readonly qualities: { value: AudioQuality; label: string }[] = [
    { value: '128', label: '128 kbps — Normal' },
    { value: '192', label: '192 kbps — Alta' },
    { value: '320', label: '320 kbps — Máxima' },
  ];

  async ngOnInit(): Promise<void> {
    const s = await this.svc.loadSettings();
    this.settings.set(s);
  }

  async onPickDir(): Promise<void> {
    const dir = await this.svc.pickOutputDirectory();
    if (dir) {
      this.settings.update((s) => ({ ...s, default_output_dir: dir }));
    }
  }

  async onSave(): Promise<void> {
    this.isSaving.set(true);
    await this.svc.saveSettings(this.settings());
    this.isSaving.set(false);
    this.saved.set(true);
    setTimeout(() => this.saved.set(false), 2500);
  }

  updateField<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void {
    this.settings.update((s) => ({ ...s, [key]: value }));
  }
}
