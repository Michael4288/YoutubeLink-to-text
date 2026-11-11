mod transcript;

use std::env;

fn main() {
    let Some(url) = env::args().nth(1) else {
        eprintln!("Uso: texto-youtube-cli <enlace-de-YouTube>");
        std::process::exit(2);
    };
    let yt_dlp = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("yt-dlp.exe")))
        .filter(|path| path.exists());
    match transcript::fetch(&url, yt_dlp.as_deref()) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}
