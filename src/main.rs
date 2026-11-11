use std::{env, fs, path::PathBuf, process::Command, time::{SystemTime, UNIX_EPOCH}};

fn main() {
    let url = env::args().nth(1).or_else(|| {
        eprintln!("Uso: texto-youtube <enlace-de-YouTube>");
        None
    });
    let Some(url) = url else { std::process::exit(2) };
    if !(url.contains("youtube.com/") || url.contains("youtu.be/")) {
        eprintln!("El enlace no parece ser de YouTube.");
        std::process::exit(2);
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = env::temp_dir().join(format!("texto-youtube-{stamp}"));
    if let Err(e) = fs::create_dir_all(&dir) { fail(format!("No pude crear carpeta temporal: {e}")); }
    let template = dir.join("sub.% (ext)s").to_string_lossy().replace("% ", "%");

    let result = Command::new("yt-dlp")
        .args(["--skip-download", "--write-subs", "--write-auto-subs", "--sub-langs", "all", "--sub-format", "vtt", "--output"])
        .arg(&template).arg(&url).output();

    let output = match result {
        Ok(output) => output,
        Err(_) => fail("No encuentro yt-dlp. Instálalo y asegúrate de que el comando yt-dlp funcione en la terminal.".into()),
    };
    if !output.status.success() {
        let _ = fs::remove_dir_all(&dir);
        fail(format!("yt-dlp no pudo obtener los subtítulos. {}", String::from_utf8_lossy(&output.stderr).trim()));
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&dir).unwrap().filter_map(Result::ok).map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "vtt")).collect();
    files.sort();
    if let Some(file) = files.first() {
        let vtt = fs::read_to_string(file).unwrap_or_default();
        let mut previous = String::new();
        for line in vtt.lines() {
            let line = line.trim();
            if line.is_empty() || line == "WEBVTT" || line.starts_with("NOTE") || line.contains("-->") || line.chars().all(|c| c.is_ascii_digit()) { continue; }
            let clean = strip_tags(line);
            if !clean.is_empty() && clean != previous { println!("{clean}"); previous = clean; }
        }
    } else {
        eprintln!("Este video no tiene subtítulos disponibles.");
    }
    let _ = fs::remove_dir_all(&dir);
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in s.chars() {
        match c { '<' => inside = true, '>' => inside = false, _ if !inside => out.push(c), _ => {} }
    }
    out.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
}

fn fail(message: String) -> ! { eprintln!("Error: {message}"); std::process::exit(1) }
