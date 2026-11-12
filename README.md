# YouTube Transcript

[Read this README in Spanish](README.es.md).

A Windows desktop app for retrieving and copying available YouTube captions. Paste a video link, choose the subtitle languages, and select **Get transcript**. The app does not download the video.

## Features

- Simple desktop window with a link field and a copyable transcript area.
- Inspect the subtitle tracks offered by a video and select one or more before retrieval. Spanish and English are selected by default when available.
- A command-line version for running transcript extraction from PowerShell.
- Deno and `yt-dlp` are downloaded into the project folder; you do not need to install them or add them to `PATH`.

## Requirements

- 64-bit Windows 10 or 11.
- PowerShell and an Internet connection for initial setup.
- GCC for Windows (MinGW-w64) available as the `gcc` command to compile the project.

Rustup uses `rsproxy.cn` as its download mirror. Local tools are stored in `.tools/` and are not installed system-wide.

## Set up and build

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
