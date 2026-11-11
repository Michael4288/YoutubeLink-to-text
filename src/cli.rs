mod transcript;

use std::env;

fn main() {
    let Some(url) = env::args().nth(1) else {
        eprintln!("Uso: texto-youtube-cli <enlace-de-YouTube> [idiomas]\nEjemplo: texto-youtube-cli <enlace> es,en");
        std::process::exit(2);
    };
    let languages = env::args().nth(2).unwrap_or_else(|| "es,en".into());
    let yt_dlp = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("yt-dlp.exe")))
        .filter(|path| path.exists());
    let deno = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("deno.exe")))
        .filter(|path| path.exists());
    match transcript::fetch(&url, &languages, yt_dlp.as_deref(), deno.as_deref()) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}
