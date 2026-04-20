import {
  Component,
  OnInit,
  inject,
  signal,
  ChangeDetectionStrategy,
} from '@angular/core';
import { CommonModule, DatePipe } from '@angular/common';
import { DownloaderService } from '../downloader/downloader.service';
import { DownloadRecord, formatDuration } from '../../models/download.models';

@Component({
  selector: 'app-history',
  standalone: true,
  imports: [CommonModule, DatePipe],
  templateUrl: './history.component.html',
  styleUrl: './history.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class HistoryComponent implements OnInit {
  private svc = inject(DownloaderService);

  records    = signal<DownloadRecord[]>([]);
  isLoading  = signal(true);
  formatDuration = formatDuration;

  async ngOnInit(): Promise<void> {
    await this.load();
  }

  async load(): Promise<void> {
    this.isLoading.set(true);
    this.records.set(await this.svc.getHistory());
    this.isLoading.set(false);
  }

  async onClear(): Promise<void> {
    await this.svc.clearHistory();
    this.records.set([]);
  }
}
