# Este directorio contiene los binarios sidecar (yt-dlp + ffmpeg).
# Los binarios NO se suben a Git (ver .gitignore).
#
# Para desarrollo local en Windows, descarga manualmente:
#
# yt-dlp:
#   https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe
#   → guardar como: yt-dlp-x86_64-pc-windows-msvc.exe
#
# ffmpeg:
#   https://github.com/BtbN/ffmpeg-builds/releases/latest
#   → descargar ffmpeg-master-latest-win64-gpl.zip
#   → extraer ffmpeg.exe y guardar como: ffmpeg-x86_64-pc-windows-msvc.exe
#
# En CI/CD (GitHub Actions) los binarios se descargan automáticamente.
