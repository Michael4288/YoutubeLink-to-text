# Texto de YouTube

[Read this README in English](README.md).

Aplicación de escritorio para Windows que obtiene y copia subtítulos disponibles de YouTube. Pega el enlace del video, elige los idiomas y pulsa **Obtener texto**. No descarga el video.

## Funciones

- Ventana sencilla con campo para el enlace y cuadro de texto copiable.
- Consulta las pistas de subtítulos que ofrece cada video y elige una o más antes de obtenerlas. Se selecciona español por defecto, o inglés si el video no tiene subtítulos en español.
- Herramienta de consola para ejecutar la extracción desde PowerShell.
- Deno y `yt-dlp` se descargan en la carpeta del proyecto; no necesitas instalarlos ni añadirlos al `PATH`.

## Requisitos

- Windows 10 u 11 de 64 bits para ejecutar la aplicación.
- Para compilar desde el código fuente: PowerShell, conexión a Internet y GCC para Windows (MinGW-w64) disponible como comando `gcc`.

Rustup usa `rsproxy.cn` como espejo de descarga. Las herramientas locales se guardan en `.tools/` y no se instalan globalmente.

## Descargar la aplicación lista para usar

Descarga el [paquete más reciente para Windows x64](https://github.com/Michael4288/YoutubeLink-to-text/releases/latest/download/texto-youtube-windows-x64.zip), extrae el ZIP y mantén sus cuatro archivos juntos en la misma carpeta. Haz doble clic en `texto-youtube.exe` para abrir la aplicación. No necesitas instalar Rust, Node.js ni Deno.

Para usar la aplicación de consola, abre PowerShell en la carpeta extraída y ejecuta:

```powershell
.\texto-youtube-cli.exe "https://www.youtube.com/watch?v=ID_DEL_VIDEO" es,en
```

## Compilar desde el código fuente

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

## Elegir idiomas de subtítulos

1. Pega un enlace de YouTube y pulsa **Buscar idiomas**.
2. La lista muestra las pistas manuales y automáticas disponibles. Se selecciona español por defecto; si no existe, inglés; y si tampoco está, la primera pista disponible.
3. Elige una o más pistas. Mantén **Ctrl** mientras haces clic para agregar o quitar pistas y luego pulsa **Obtener texto**. Cada pista obtenida aparece en el resultado con su código de idioma.

La aplicación consulta qué pistas existen antes de descargar subtítulos. No descarga todas las pistas automáticamente. Para reducir errores por límite de solicitudes, espera entre solicitudes y reintenta las solicitudes HTTP fallidas. Si YouTube responde HTTP 429, espera unos minutos antes de volver a intentarlo.

## Usar la consola

El argumento de idiomas es opcional. Si lo omites, se solicitan subtítulos en español e inglés:

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
