# Texto de YouTube

Programa sencillo en Rust: recibe un enlace de YouTube y muestra la transcripción disponible, sin descargar el video.

## Requisito

Instala [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation) y comprueba que el comando `yt-dlp` funcione desde una terminal. El programa usa subtítulos publicados por el canal o subtítulos automáticos. Si el video no ofrece subtítulos, no puede inventar una transcripción.

## Ejecutar

Desde esta carpeta:

```sh
cargo run -- "https://www.youtube.com/watch?v=ID_DEL_VIDEO"
```

Para crear un ejecutable:

```sh
cargo build --release
```

La transcripción sale por la terminal. El programa usa el primer archivo de subtítulos que yt-dlp entrega y elimina los tiempos y etiquetas de formato.
