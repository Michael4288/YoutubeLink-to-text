# Texto de YouTube

Aplicación de escritorio para obtener y copiar subtítulos disponibles en YouTube. Pega el enlace del video, elige los idiomas y pulsa **Obtener texto**. No descarga el video.

## Funciones

- Ventana sencilla con campo para el enlace y cuadro de texto copiable.
- Español e inglés (`es,en`) como idiomas iniciales; puedes indicar otros códigos separados por comas, por ejemplo `fr,ja`.
- Herramienta de consola incluida para ejecutar la extracción desde PowerShell.
- Deno y `yt-dlp` se descargan en la carpeta del proyecto; no necesitas instalarlos ni añadirlos al `PATH`.

## Requisitos

- Windows 10 u 11 de 64 bits.
- PowerShell y conexión a Internet para la configuración inicial.
- GCC para Windows (MinGW-w64) disponible como comando `gcc` para compilar.

La configuración local de Rust usa `rsproxy.cn` como espejo de descarga. Las herramientas se guardan en `.tools/` y no se instalan globalmente.

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

Mantén juntos estos cuatro archivos al compartir la aplicación: `texto-youtube.exe`, `texto-youtube-cli.exe`, `yt-dlp.exe` y `deno.exe`.

## Usar la consola

El último argumento es opcional; si se omite, se buscan subtítulos en español e inglés:

```powershell
.\target\release\texto-youtube-cli.exe "https://www.youtube.com/watch?v=ID_DEL_VIDEO" es,en
```

## Límites y solución de problemas

- Solo se pueden obtener subtítulos publicados por el canal o generados automáticamente. Si no hay subtítulos para los idiomas elegidos, la aplicación no puede crear una transcripción.
- Si aparece HTTP 429, YouTube limitó temporalmente las solicitudes. Espera unos minutos antes de volver a intentar y solicita solo los idiomas necesarios.
- Si faltan `yt-dlp.exe` o `deno.exe`, ejecuta `setup.ps1` y luego `build.ps1` otra vez.
- Si la configuración no puede descargar herramientas, comprueba la conexión a Internet y vuelve a ejecutar `setup.ps1`.

## Archivos de trabajo

`.tools/` contiene herramientas locales y `target/` contiene los ejecutables compilados. Git ignora ambas carpetas; `Cargo.lock` se conserva en el repositorio.
