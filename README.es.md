# Texto de YouTube

[Read this README in English](README.md).

Aplicación de escritorio para Windows que obtiene y copia subtítulos disponibles de YouTube. Pega el enlace del video, elige los idiomas y pulsa **Obtener texto**. No descarga el video.

## Funciones

- Ventana sencilla con campo para el enlace y cuadro de texto copiable.
- Español e inglés (`es,en`) como idiomas iniciales; puedes indicar otros códigos separados por comas, por ejemplo `fr,ja`.
- Herramienta de consola para ejecutar la extracción desde PowerShell.
- Deno y `yt-dlp` se descargan en la carpeta del proyecto; no necesitas instalarlos ni añadirlos al `PATH`.

## Requisitos

- Windows 10 u 11 de 64 bits.
- PowerShell y conexión a Internet para la configuración inicial.
- GCC para Windows (MinGW-w64) disponible como comando `gcc` para compilar.

Rustup usa `rsproxy.cn` como espejo de descarga. Las herramientas locales se guardan en `.tools/` y no se instalan globalmente.

## Configurar y compilar

Abre PowerShell en la carpeta del proyecto y ejecuta:

```powershell
.\setup.ps1
.\build.ps1
```

La configuración descarga Rust, `yt-dlp` y Deno dentro de `.tools/`. La compilación coloca los ejecutables en `target\release`.

## Abrir la aplicación

Haz doble clic en `target\release\texto-youtube.exe`, o ejecútalo desde PowerShell:

```powershell
.\target\release\texto-youtube.exe
```

Al compartir la aplicación, mantén juntos estos cuatro archivos: `texto-youtube.exe`, `texto-youtube-cli.exe`, `yt-dlp.exe` y `deno.exe`.

## Usar la consola

El argumento de idiomas es opcional. Si lo omites, se buscan subtítulos en español e inglés:

```powershell
.\target\release\texto-youtube-cli.exe "https://www.youtube.com/watch?v=ID_DEL_VIDEO" es,en
```

## Límites y solución de problemas

- Solo se pueden obtener subtítulos publicados por el canal o generados automáticamente. Si no hay subtítulos en los idiomas elegidos, la aplicación no puede crear una transcripción.
- Si YouTube devuelve HTTP 429, limitó temporalmente las solicitudes. Espera unos minutos antes de volver a intentar y solicita solo los idiomas necesarios.
- Si falta `yt-dlp.exe` o `deno.exe`, ejecuta `setup.ps1` y luego `build.ps1` otra vez.
- Si la configuración no puede descargar las herramientas, comprueba la conexión a Internet y vuelve a ejecutar `setup.ps1`.

## Archivos locales

`.tools/` contiene las herramientas locales y `target/` los ejecutables compilados. Git ignora ambas carpetas; `Cargo.lock` se conserva en el repositorio.
