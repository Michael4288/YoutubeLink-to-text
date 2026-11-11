use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn fetch(url: &str, yt_dlp: Option<&Path>, deno: Option<&Path>) -> Result<String, String> {
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
    let Some(deno) = deno else {
        let _ = fs::remove_dir_all(&dir);
        return Err("No encuentro Deno junto a la aplicación. Vuelve a ejecutar setup.ps1 y build.ps1.".into());
    };

    let result = Command::new(executable)
        .arg("--js-runtimes")
        .arg(format!("deno:{}", deno.display()))
        .args([
            "--skip-download",
            "--write-subs",
            "--write-auto-subs",
            "--sub-langs",
            "all",
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
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let _ = fs::remove_dir_all(&dir);
        return Err(format!("yt-dlp no pudo obtener los subtítulos. {message}"));
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("No pude leer los subtítulos: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vtt"))
        .collect();
    files.sort();

    let transcript = if let Some(file) = files.first() {
        let vtt = fs::read_to_string(file).map_err(|e| format!("No pude leer la transcripción: {e}"))?;
        clean_vtt(&vtt)
    } else {
        String::new()
    };
    let _ = fs::remove_dir_all(&dir);

    if transcript.is_empty() {
        Err("Este video no tiene subtítulos disponibles.".into())
    } else {
        Ok(transcript)
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
