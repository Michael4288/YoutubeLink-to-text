use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug)]
pub struct SubtitleTrack {
    pub code: String,
    pub name: String,
    pub source: String,
    manual: bool,
}

impl SubtitleTrack {
    pub fn display_name(&self) -> String {
        format!("{} — {} ({})", self.code, self.name, self.source)
    }
}

pub fn discover(url: &str, yt_dlp: Option<&Path>, deno: Option<&Path>) -> Result<Vec<SubtitleTrack>, String> {
    if !(url.contains("youtube.com/") || url.contains("youtu.be/")) {
        return Err("El enlace no parece ser de YouTube.".into());
    }
    let executable = yt_dlp.map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("yt-dlp"));
    let Some(deno) = deno else {
        return Err("No encuentro Deno junto a la aplicación. Vuelve a ejecutar setup.ps1 y build.ps1.".into());
    };
    let output = Command::new(executable)
        .arg("--js-runtimes")
        .arg(format!("deno:{}", deno.display()))
        .args([
            "--sleep-requests",
            "1",
            "--retry-sleep",
            "http:exp=2:30",
            "--skip-download",
            "--list-subs",
            url,
        ])
        .output()
        .map_err(|e| format!("No pude iniciar yt-dlp: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let message = format!("{stderr}\n{stdout}");
        if message.contains("429") || message.contains("Too Many Requests") {
            return Err("YouTube limitó temporalmente las solicitudes (HTTP 429). Espera unos minutos y vuelve a intentar.".into());
        }
        return Err(format!("No pude consultar los idiomas. {}", stderr.trim()));
    }
    let tracks = parse_tracks(&stdout);
    if tracks.is_empty() {
        Err("Este video no ofrece subtítulos disponibles.".into())
    } else {
        Ok(tracks)
    }
}

fn parse_tracks(output: &str) -> Vec<SubtitleTrack> {
    let mut section = "";
    let mut tracks: Vec<SubtitleTrack> = Vec::new();
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("available automatic captions for") {
            section = "automatic";
            continue;
        }
        if lower.contains("available subtitles for") {
            section = "manual";
            continue;
        }
        if section.is_empty() || lower.contains("language") && lower.contains("format") {
            continue;
        }
        let Some(code) = line.split_whitespace().next() else { continue };
        if !valid_language_code(code) {
            continue;
        }
        let Some(position) = line.find(code) else { continue };
        let remainder = &line[position + code.len()..];
        let name = if section == "automatic" {
            remainder.split(',').next().unwrap_or_default().trim().to_owned()
        } else {
            remainder
                .split_whitespace()
                .take_while(|word| !word.contains(','))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let name = if name.is_empty() { code.to_owned() } else { name };
        let manual = section == "manual";
        if let Some(existing) = tracks.iter_mut().find(|track| track.code == code) {
            if manual && !existing.manual {
                existing.name = name;
                existing.source = "manual + automático".into();
                existing.manual = true;
            } else if !manual && existing.manual {
                existing.source = "manual + automático".into();
            }
            continue;
        }
        tracks.push(SubtitleTrack {
            code: code.to_owned(),
            name,
            source: if manual { "manual" } else { "automático" }.into(),
            manual,
        });
    }
    tracks.sort_by(|a, b| {
        let rank = |code: &str| match code { "es" => 0, "en" => 1, _ => 2 };
        rank(&a.code).cmp(&rank(&b.code)).then_with(|| a.name.cmp(&b.name))
    });
    tracks
}

fn valid_language_code(code: &str) -> bool {
    !code.is_empty()
        && code.chars().any(|c| c.is_ascii_alphabetic())
        && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn fetch(url: &str, languages: &str, yt_dlp: Option<&Path>, deno: Option<&Path>) -> Result<String, String> {
    if !(url.contains("youtube.com/") || url.contains("youtu.be/")) {
        return Err("El enlace no parece ser de YouTube.".into());
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let dir = env::temp_dir().join(format!("texto-youtube-{stamp}"));
    fs::create_dir_all(&dir).map_err(|e| format!("No pude crear carpeta temporal: {e}"))?;
    let template = dir.join("sub.%(ext)s");
    let executable = yt_dlp.map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("yt-dlp"));
    let languages = languages.split(',').map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(",");
    if languages.is_empty() {
        let _ = fs::remove_dir_all(&dir);
        return Err("Escribe al menos un idioma, por ejemplo: es,en".into());
    }
    let Some(deno) = deno else {
        let _ = fs::remove_dir_all(&dir);
        return Err("No encuentro Deno junto a la aplicación. Vuelve a ejecutar setup.ps1 y build.ps1.".into());
    };

    let result = Command::new(executable)
        .arg("--js-runtimes")
        .arg(format!("deno:{}", deno.display()))
        .args([
            "--sleep-requests",
            "1",
            "--sleep-subtitles",
            "2",
            "--retry-sleep",
            "http:exp=2:30",
            "--skip-download",
            "--write-subs",
            "--write-auto-subs",
            "--sub-langs",
            &languages,
            "--sub-format",
            "vtt",
            "--output",
        ])
        .arg(&template)
        .arg(url)
        .output();

    let output = match result {
        Ok(output) => output,
        Err(e) => {
            let _ = fs::remove_dir_all(&dir);
            return Err(format!("No pude iniciar yt-dlp: {e}"));
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_dir_all(&dir);
        if stderr.contains("429") || stderr.contains("Too Many Requests") {
            return Err("YouTube limitó temporalmente las solicitudes (HTTP 429). Espera unos minutos y vuelve a intentar.".into());
        }
        let message = stderr.lines().filter(|line| !line.contains("WARNING: ffmpeg not found")).collect::<Vec<_>>().join("\r\n").trim().to_owned();
        return Err(format!("yt-dlp no pudo obtener los subtítulos. {message}"));
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("No pude leer los subtítulos: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vtt"))
        .collect();
    files.sort();

    let mut transcripts = Vec::new();
    for file in &files {
        let vtt = match fs::read_to_string(file) {
            Ok(vtt) => vtt,
            Err(e) => {
                let _ = fs::remove_dir_all(&dir);
                return Err(format!("No pude leer la transcripción: {e}"));
            }
        };
        let text = clean_vtt(&vtt);
        if !text.is_empty() {
            let language = file.file_stem().and_then(|stem| stem.to_str())
                .and_then(|stem| stem.strip_prefix("sub."))
                .unwrap_or("subtítulo");
            transcripts.push(format!("===== {language} =====\r\n{text}"));
        }
    }
    let transcript = transcripts.join("\r\n\r\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let had_rate_limit = stderr.contains("429") || stderr.contains("Too Many Requests");
    let _ = fs::remove_dir_all(&dir);

    if transcript.is_empty() {
        if had_rate_limit {
            Err("YouTube limitó temporalmente las solicitudes (HTTP 429). Espera unos minutos y vuelve a intentar.".into())
        } else {
            Err("Este video no tiene subtítulos disponibles.".into())
        }
    } else {
        if had_rate_limit {
            Ok(format!("Algunas pistas no se pudieron obtener porque YouTube respondió HTTP 429. Espera unos minutos para volver a intentarlo.\r\n\r\n{transcript}"))
        } else {
            Ok(transcript)
        }
    }
}

fn clean_vtt(vtt: &str) -> String {
    let mut lines = Vec::new();
    let mut previous = String::new();
    for line in vtt.lines() {
        let line = line.trim();
        if line.is_empty()
            || line == "WEBVTT"
            || line.starts_with("NOTE")
            || line.contains("-->")
            || line.chars().all(|c| c.is_ascii_digit())
        {
            continue;
        }
        let clean = strip_tags(line);
        if !clean.is_empty() && clean != previous {
            lines.push(clean.clone());
            previous = clean;
        }
    }
    lines.join("\r\n")
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in s.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}
