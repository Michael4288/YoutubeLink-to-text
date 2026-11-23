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

const VALID_YOUTUBE_EXAMPLE: &str = "https://www.youtube.com/watch?v=VIDEO_ID";

fn normalize_youtube_url(input: &str) -> Result<String, String> {
    let invalid = || {
        format!(
            "El enlace no es válido. Usa una URL de un video de YouTube, por ejemplo: {VALID_YOUTUBE_EXAMPLE}"
        )
    };
    let input = input.trim();
    if input.is_empty() || input.chars().any(char::is_whitespace) {
        return Err(invalid());
    }

    let (scheme, remainder) = input.split_once("://").ok_or_else(invalid)?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(invalid());
    }

    let authority_end = remainder
        .find(|c| matches!(c, '/' | '?' | '#'))
        .unwrap_or(remainder.len());
    let authority = &remainder[..authority_end];
    if authority.is_empty() || authority.contains('@') || authority.contains('%') {
        return Err(invalid());
    }

    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    let host = host.to_ascii_lowercase();
    let host = host.strip_suffix('.').unwrap_or(&host);
    if let Some(port) = port {
        let expected_port = if scheme == "https" { "443" } else { "80" };
        if port != expected_port {
            return Err(invalid());
        }
    }

    let valid_domain = host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    });
    if !valid_domain {
        return Err(invalid());
    }
    let is_short_domain = host == "youtu.be";
    let is_youtube_domain = host == "youtube.com" || host.ends_with(".youtube.com");
    if !is_short_domain && !is_youtube_domain {
        return Err(invalid());
    }

    let tail = &remainder[authority_end..];
    let tail = tail.split('#').next().unwrap_or_default();
    let (path, query) = match tail.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (tail, None),
    };
    let path = path.strip_suffix('/').unwrap_or(path);
    let video_id = if is_short_domain {
        let Some(id) = path.strip_prefix('/') else {
            return Err(invalid());
        };
        if id.is_empty() || id.contains('/') {
            return Err(invalid());
        }
        id
    } else if path == "/watch" {
        let mut found = None;
        for parameter in query.unwrap_or_default().split('&') {
            let (key, value) = parameter.split_once('=').unwrap_or((parameter, ""));
            if key == "v" {
                if found.is_some() {
                    return Err(invalid());
                }
                found = Some(value);
            }
        }
        found.filter(|id| !id.is_empty()).ok_or_else(invalid)?
    } else {
        let Some(segments) = path.strip_prefix('/') else {
            return Err(invalid());
        };
        let mut segments = segments.split('/');
        let route = segments.next().unwrap_or_default();
        let id = segments.next().unwrap_or_default();
        if !matches!(route, "shorts" | "live" | "embed")
            || id.is_empty()
            || segments.next().is_some()
        {
            return Err(invalid());
        }
        id
    };

    if video_id == "."
        || video_id == ".."
        || !video_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '~'))
    {
        return Err(invalid());
    }

    Ok(format!("https://www.youtube.com/watch?v={video_id}"))
}

impl SubtitleTrack {
    pub fn display_name(&self) -> String {
        format!("{} — {} ({})", self.code, self.name, self.source)
    }
}

pub fn discover(
    url: &str,
    yt_dlp: Option<&Path>,
    deno: Option<&Path>,
) -> Result<Vec<SubtitleTrack>, String> {
    let normalized_url = normalize_youtube_url(url)?;
    let executable = yt_dlp
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("yt-dlp"));
    let Some(deno) = deno else {
        return Err(
            "No encuentro Deno junto a la aplicación. Vuelve a ejecutar setup.ps1 y build.ps1."
                .into(),
        );
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
        ])
        .arg(&normalized_url)
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
        let Some(code) = line.split_whitespace().next() else {
            continue;
        };
        if !valid_language_code(code) {
            continue;
        }
        let Some(position) = line.find(code) else {
            continue;
        };
        let remainder = &line[position + code.len()..];
        let name = if section == "automatic" {
            remainder
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned()
        } else {
            remainder
                .split_whitespace()
                .take_while(|word| !word.contains(','))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let name = if name.is_empty() {
            code.to_owned()
        } else {
            name
        };
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
        let rank = |code: &str| match code {
            "es" => 0,
            "en" => 1,
            _ => 2,
        };
        rank(&a.code)
            .cmp(&rank(&b.code))
            .then_with(|| a.name.cmp(&b.name))
    });
    tracks
}

fn valid_language_code(code: &str) -> bool {
    !code.is_empty()
        && code.chars().any(|c| c.is_ascii_alphabetic())
        && code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn fetch(
    url: &str,
    languages: &str,
    yt_dlp: Option<&Path>,
    deno: Option<&Path>,
) -> Result<String, String> {
    let normalized_url = normalize_youtube_url(url)?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let dir = env::temp_dir().join(format!("texto-youtube-{stamp}"));
    fs::create_dir_all(&dir).map_err(|e| format!("No pude crear carpeta temporal: {e}"))?;
    let template = dir.join("sub.%(ext)s");
    let executable = yt_dlp
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("yt-dlp"));
    let languages = languages
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    if languages.is_empty() {
        let _ = fs::remove_dir_all(&dir);
        return Err("Escribe al menos un idioma, por ejemplo: es,en".into());
    }
    let Some(deno) = deno else {
        let _ = fs::remove_dir_all(&dir);
        return Err(
            "No encuentro Deno junto a la aplicación. Vuelve a ejecutar setup.ps1 y build.ps1."
                .into(),
        );
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
        .arg(&normalized_url)
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
        let message = stderr
            .lines()
            .filter(|line| !line.contains("WARNING: ffmpeg not found"))
            .collect::<Vec<_>>()
            .join("\r\n")
            .trim()
            .to_owned();
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
            let language = file
                .file_stem()
                .and_then(|stem| stem.to_str())
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
        let clean = decode_vtt_entities(&strip_tags(line));
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
    out
}

fn decode_vtt_entities(text: &str) -> String {
    let mut output = String::new();
    let mut remaining = text;
    while let Some(start) = remaining.find('&') {
        output.push_str(&remaining[..start]);
        remaining = &remaining[start..];
        let Some(end) = remaining.find(';') else {
            output.push_str(remaining);
            return output;
        };
        let entity = &remaining[1..end];
        if let Some(decoded) = decode_vtt_entity(entity) {
            output.push(decoded);
        } else {
            output.push_str(&remaining[..=end]);
        }
        remaining = &remaining[end + 1..];
    }
    output.push_str(remaining);
    output
}

fn decode_vtt_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "nbsp" => Some('\u{00a0}'),
        _ => {
            let (radix, digits) = if let Some(hex) = entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
            {
                (16, hex)
            } else if let Some(decimal) = entity.strip_prefix('#') {
                (10, decimal)
            } else {
                return None;
            };
            let codepoint = u32::from_str_radix(digits, radix).ok()?;
            if codepoint == 0 {
                return None;
            }
            char::from_u32(codepoint)
        }
    }
}

#[cfg(test)]
mod vtt_entity_tests {
    use super::{clean_vtt, decode_vtt_entities};

    #[test]
    fn decodes_named_decimal_and_hex_entities_in_mixed_unicode_text() {
        assert_eq!(
            decode_vtt_entities(
                "&amp; &lt; &gt; &quot; &apos; &#39; &nbsp; &#233; &#x1F600; &#X41; café 漢字"
            ),
            "& < > \" ' ' \u{00a0} é 😀 A café 漢字"
        );
    }

    #[test]
    fn leaves_unknown_invalid_and_unterminated_entities_unchanged() {
        let value = "&unknown; &#; &#xZZ; &#xD800; &#0; &#1114112; &missing";
        assert_eq!(decode_vtt_entities(value), value);
    }

    #[test]
    fn decodes_once_after_tag_stripping_and_before_adjacent_deduplication() {
        let vtt = "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n<b>&amp;lt;</b>\n\n00:00:01.000 --> 00:00:02.000\n&lt;\n";
        assert_eq!(clean_vtt(vtt), "&lt;\r\n<");
        let decoded = "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nhi&amp;\n\n00:00:01.000 --> 00:00:02.000\nhi&\n";
        assert_eq!(clean_vtt(decoded), "hi&");
    }
}

#[cfg(test)]
mod url_validation_tests {
    use super::{normalize_youtube_url, VALID_YOUTUBE_EXAMPLE};

    #[test]
    fn accepts_supported_video_url_forms_and_normalizes_them() {
        let cases = [
            (
                "  http://youtube.com/watch?v=abc_123&feature=share  ",
                "abc_123",
            ),
            (
                "https://music.youtube.com/watch?feature=share&v=short-id#t=2",
                "short-id",
            ),
            ("https://youtu.be/another.id/", "another.id"),
            ("https://www.youtube.com/shorts/a", "a"),
            ("https://www.youtube.com/live/video~id", "video~id"),
            ("https://www.youtube.com/embed/Video_ID", "Video_ID"),
            ("https://youtube.com:443/watch?v=abc", "abc"),
        ];

        for (input, video_id) in cases {
            assert_eq!(
                normalize_youtube_url(input).unwrap(),
                format!("https://www.youtube.com/watch?v={video_id}"),
                "input: {input}"
            );
        }
    }

    #[test]
    fn rejects_non_video_or_misleading_urls() {
        let cases = [
            "https://attacker.example/watch?v=abc&next=youtube.com/",
            "https://youtube.com.attacker.example/watch?v=abc",
            "https://.youtube.com/watch?v=abc",
            "https://evil..youtube.com/watch?v=abc",
            "https://youtube.com@attacker.example/watch?v=abc",
            "https://youtube.com/playlist?list=abc",
            "https://youtube.com/watch",
            "https://youtube.com/watch?v=",
            "https://youtube.com/watch?v=abc&v=def",
            "https://youtube.com/shorts/abc/extra",
            "ftp://youtube.com/watch?v=abc",
            "https://youtube.com:8443/watch?v=abc",
            "https://youtube.com/watch?v=abc def",
            "https://youtube.com/watch?v=abc\nhttps://youtu.be/def",
            "https://youtu.be/../secret",
        ];

        for input in cases {
            let error = normalize_youtube_url(input).unwrap_err();
            assert!(error.contains(VALID_YOUTUBE_EXAMPLE));
            assert!(error.contains("no es válido"));
        }
    }
}
