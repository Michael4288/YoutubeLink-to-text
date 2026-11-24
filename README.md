# YouTube Transcript

[Read this README in Spanish](README.es.md).

A Windows desktop app for retrieving and copying available YouTube captions. Paste a video link, choose the subtitle languages, and select **Get transcript**. The app does not download the video.

## Features

- Simple desktop window with a link field and a copyable transcript area.
- Inspect the subtitle tracks offered by a video and select one or more before retrieval. Spanish is selected by default, then English if Spanish is unavailable.
- A command-line version for running transcript extraction from PowerShell.
- Deno and `yt-dlp` are downloaded into the project folder; you do not need to install them or add them to `PATH`.

The app accepts HTTP or HTTPS video links using YouTube's `watch`, `shorts`, `live`, and `embed` formats, including YouTube subdomains, as well as `youtu.be` links. Playlist-only links and other hosts are rejected before `yt-dlp` starts.

## Requirements

- 64-bit Windows 10 or 11 to run the app.
- To build from source: PowerShell, an Internet connection, and GCC for Windows (MinGW-w64) available as the `gcc` command.

Rustup uses `rsproxy.cn` as its download mirror. Local tools are stored in `.tools/` and are not installed system-wide.

## Download the ready-to-run app

Download the [latest Windows x64 package](https://github.com/Michael4288/YoutubeLink-to-text/releases/latest/download/texto-youtube-windows-x64.zip), extract the ZIP, and keep its four files together in the same folder. Double-click `texto-youtube.exe` to open the app. You do not need to install Rust, Node.js, or Deno.

To use the command-line app, open PowerShell in the extracted folder and run:

```powershell
.\texto-youtube-cli.exe "https://www.youtube.com/watch?v=VIDEO_ID" es,en
```

## Build from source

Open PowerShell in the project folder and run:

```powershell
.\setup.ps1
.\build.ps1
```

Setup downloads Rust, `yt-dlp`, and Deno into `.tools/`. The build places the executables in `target\release`.

## Launch the app

Double-click `target\release\texto-youtube.exe`, or run it from PowerShell:

```powershell
.\target\release\texto-youtube.exe
```

Keep these four files together when sharing the app: `texto-youtube.exe`, `texto-youtube-cli.exe`, `yt-dlp.exe`, and `deno.exe`.

## Choose subtitle languages

1. Paste a YouTube link and select **Search languages**.
2. The list shows the available manual and automatic caption tracks. Spanish is selected by default; if unavailable, English is selected; otherwise the first available track is selected.
3. Select one or more tracks. Hold **Ctrl** while clicking to add or remove tracks, then select **Get transcript**. Each retrieved track appears in the result with its language code.

When retrieval starts, the app remembers the selected language codes in `%APPDATA%\TextoYoutube\preferences.json`. On the next search it preselects all remembered languages available for that video. If none are available, it falls back to Spanish, then English, then the first track. Delete `preferences.json` to reset this preference; it stores language codes only.

The app checks which tracks are available before downloading captions. It does not download every track automatically. To reduce request limit errors, it spaces out requests and retries failed HTTP requests. If YouTube returns HTTP 429, wait a few minutes before trying again.

## Use the command line

The language argument is optional. If omitted, the app requests Spanish and English subtitles:

```powershell
.\target\release\texto-youtube-cli.exe "https://www.youtube.com/watch?v=VIDEO_ID" es,en
```

## Limitations and troubleshooting

- The app can only retrieve captions published by the channel or generated automatically. It cannot create a transcript when no captions are available for the selected languages.
- If YouTube returns HTTP 429, it has temporarily limited requests. Wait a few minutes before trying again and request only the languages you need.
- If `yt-dlp.exe` or `deno.exe` is missing, run `setup.ps1` and then `build.ps1` again.
- If setup cannot download the tools, check your Internet connection and run `setup.ps1` again.

## Local build files

`.tools/` contains local tools and `target/` contains compiled executables. Git ignores both folders; `Cargo.lock` is tracked in the repository.
