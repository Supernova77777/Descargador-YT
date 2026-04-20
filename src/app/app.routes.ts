import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'download',
    pathMatch: 'full',
  },
  {
    path: 'download',
    loadComponent: () =>
      import('./features/downloader/downloader.component').then(
        (m) => m.DownloaderComponent
      ),
  },
  {
    path: 'history',
    loadComponent: () =>
      import('./features/history/history.component').then(
        (m) => m.HistoryComponent
      ),
  },
  {
    path: 'settings',
    loadComponent: () =>
      import('./features/settings/settings.component').then(
        (m) => m.SettingsComponent
      ),
  },
  { path: '**', redirectTo: 'download' },
];
