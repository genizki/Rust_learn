# Rust_learn
Dies ist ein Hobby Projekt, das dazu dient, die Programmiersprache Rust zu lernen und praktische Erfahrungen zu sammeln. Jegliche Beiträge und Experimente in diesem Repository erfolgen zu Lernzwecken und sind nicht für den produktiven Einsatz gedacht.

## Installation und Ausführung

### Voraussetzungen

Um dieses Projekt auszuführen, müssen folgende Schritte durchgeführt werden:

### 1. yt-dlp Binary herunterladen

Lade das passende yt-dlp Binary für dein Betriebssystem herunter:

- **Windows**: Lade die `.exe` Datei von der [offiziellen yt-dlp Release-Seite](https://github.com/yt-dlp/yt-dlp/releases) herunter
- **Mac**: Lade die macOS Binary von der [offiziellen yt-dlp Release-Seite](https://github.com/yt-dlp/yt-dlp/releases) herunter

Speichere die heruntergeladene Datei im Ordner `yt_dlp` im Projektverzeichnis.

### 2. ffmpeg und ffprobe installieren

Lade ffmpeg und ffprobe für dein Betriebssystem herunter:

- **Windows**: Lade die Windows Builds von [ffmpeg.org](https://ffmpeg.org/download.html) herunter
- **Mac**: Lade die macOS Builds von [ffmpeg.org](https://ffmpeg.org/download.html) herunter oder installiere sie via Homebrew: `brew install ffmpeg`

Speichere beide Binaries (`ffmpeg` und `ffprobe`) im Ordner `ffmpeg` im Projektverzeichnis.

### 3. Berechtigungen für Binaries setzen

**Wichtig**: Die Binaries müssen ausführbar sein, damit sie vom Programm verwendet werden können.

#### Auf Mac:

Öffne ein Terminal im Projektverzeichnis und führe folgende Befehle aus:

```bash
# Berechtigungen für yt-dlp setzen
chmod +x yt_dlp/yt-dlp_macos

# Berechtigungen für ffmpeg und ffprobe setzen
chmod +x ffmpeg/ffmpeg
chmod +x ffmpeg/ffprobe
```

#### Auf Windows:

Unter Windows sollten `.exe` Dateien standardmäßig ausführbar sein. Falls es Probleme gibt, überprüfe:
- Dass die Dateien nicht blockiert sind (Rechtsklick → Eigenschaften → "Zulassen" aktivieren)
- Dass dein Antivirenprogramm die Ausführung nicht blockiert

### 4. Projekt ausführen

Nachdem alle Voraussetzungen erfüllt sind, kann das Projekt mit folgendem Befehl ausgeführt werden:

```bash
cargo run
```

## Ordnerstruktur

Stelle sicher, dass folgende Ordnerstruktur vorhanden ist:

```
rust-journey/
├── yt_dlp/
│   └── yt-dlp_macos (oder yt-dlp.exe)
├── ffmpeg/
│   ├── ffmpeg (oder ffmpeg.exe)
│   └── ffprobe (oder ffprobe.exe)
└── src/...
```
