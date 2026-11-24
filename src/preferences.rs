use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const SCHEMA_VERSION: u32 = 1;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
struct Preferences {
    version: u32,
    languages: Vec<String>,
}

fn preferences_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(|root| {
        PathBuf::from(root)
            .join("TextoYoutube")
            .join("preferences.json")
    })
}

pub fn load_selected_languages() -> Vec<String> {
    preferences_path()
        .map(|path| load_from(&path))
        .unwrap_or_default()
}

fn load_from(path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(preferences) = serde_json::from_str::<Preferences>(&contents) else {
        return Vec::new();
    };
    if preferences.version != SCHEMA_VERSION {
        return Vec::new();
    }
    let normalized = preferences
        .languages
        .iter()
        .map(|code| normalize_code(code))
        .collect::<Option<Vec<_>>>();
    normalized.unwrap_or_default()
}

pub fn save_selected_languages(codes: &str) -> Result<(), String> {
    let path = preferences_path()
        .ok_or_else(|| "No se pudo encontrar la carpeta de datos del usuario.".to_owned())?;
    let languages = codes
        .split(',')
        .map(normalize_code)
        .collect::<Option<Vec<_>>>();
    let Some(languages) = languages.filter(|codes| !codes.is_empty()) else {
        return Err(
            "No se guardaron las preferencias porque los códigos de idioma no son válidos.".into(),
        );
    };
    save_to(
        &path,
        &Preferences {
            version: SCHEMA_VERSION,
            languages,
        },
    )
    .map_err(|error| format!("No se pudieron guardar los idiomas preferidos: {error}"))
}

fn normalize_code(code: &str) -> Option<String> {
    let code = code.trim();
    let segments = code.split(['-', '_']).collect::<Vec<_>>();
    if code.len() > 64
        || segments.is_empty()
        || !segments[0].chars().all(|c| c.is_ascii_alphabetic())
        || segments.iter().any(|segment| {
            segment.is_empty() || !segment.chars().all(|c| c.is_ascii_alphanumeric())
        })
    {
        return None;
    }
    Some(code.replace('_', "-").to_ascii_lowercase())
}

fn save_to(path: &Path, preferences: &Preferences) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("ruta de preferencias sin carpeta"))?;
    fs::create_dir_all(parent)?;
    let contents = serde_json::to_vec(preferences).map_err(io::Error::other)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(
        "preferences.{}.{}.{}.tmp",
        std::process::id(),
        stamp,
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&contents)?;
        file.sync_all()?;
        replace_file(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let source = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let success = unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0x1 | 0x8) };
    if success == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

pub fn selected_track_indices(offered: &[String], remembered: &[String]) -> Vec<usize> {
    let remembered = remembered
        .iter()
        .filter_map(|code| normalize_code(code))
        .collect::<Vec<_>>();
    let selected = offered
        .iter()
        .enumerate()
        .filter_map(|(index, code)| {
            normalize_code(code)
                .filter(|normalized| remembered.contains(normalized))
                .map(|_| index)
        })
        .collect::<Vec<_>>();
    if !selected.is_empty() {
        return selected;
    }
    offered
        .iter()
        .position(|code| normalize_code(code).as_deref() == Some("es"))
        .or_else(|| {
            offered
                .iter()
                .position(|code| normalize_code(code).as_deref() == Some("en"))
        })
        .or_else(|| (!offered.is_empty()).then_some(0))
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path() -> PathBuf {
        env::temp_dir()
            .join(format!(
                "texto-youtube-preferences-test-{}-{}",
                std::process::id(),
                TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
            ))
            .join("preferences.json")
    }

    #[test]
    fn loads_valid_codes_ignores_unknown_fields_and_rejects_bad_versions() {
        let path = temp_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"{"version":1,"languages":["FR_ca","en"],"future":true}"#,
        )
        .unwrap();
        assert_eq!(load_from(&path), ["fr-ca", "en"]);
        fs::write(&path, r#"{"version":2,"languages":["es"]}"#).unwrap();
        assert!(load_from(&path).is_empty());
        fs::write(&path, "not json").unwrap();
        assert!(load_from(&path).is_empty());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn saves_atomically_and_only_stores_language_codes() {
        let path = temp_path();
        let value = Preferences {
            version: SCHEMA_VERSION,
            languages: vec!["fr".into()],
        };
        save_to(&path, &value).unwrap();
        save_to(
            &path,
            &Preferences {
                version: SCHEMA_VERSION,
                languages: vec!["de".into()],
            },
        )
        .unwrap();
        assert_eq!(load_from(&path), ["de"]);
        assert!(!fs::read_to_string(&path).unwrap().contains("transcript"));
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn restores_all_matching_languages_or_uses_the_existing_fallback() {
        let offered = vec!["es".into(), "fr".into(), "en-US".into(), "fr-CA".into()];
        assert_eq!(
            selected_track_indices(&offered, &["EN_us".into(), "fr-ca".into()]),
            [2, 3]
        );
        assert_eq!(selected_track_indices(&offered, &["ja".into()]), [0]);
        assert_eq!(
            selected_track_indices(&["de".into(), "en".into()], &[]),
            [1]
        );
    }
}
