# Texto de YouTube — versión 2

Aplicación de escritorio para Windows que obtiene transcripciones disponibles de YouTube sin descargar el video. Pega el enlace, pulsa **Obtener texto** y copia el resultado desde la ventana.

## Requisitos

- Windows 10/11 de 64 bits y PowerShell.
- Conexión a Internet durante la configuración inicial.
- GCC para Windows (MinGW-w64) disponible como comando `gcc`.

No necesitas tener Node ni Deno instalados. La configuración descarga Rust, `yt-dlp` y Deno dentro de `.tools/`; no modifica el `PATH` ni instala herramientas globalmente. Rustup usa `rsproxy.cn` como espejo de descarga del toolchain.

## Configurar

Abre PowerShell en la carpeta del proyecto y ejecuta:

```powershell
.\setup.ps1
```

El script descarga Rust (toolchain GNU), `yt-dlp` y Deno para Windows x64 en `.tools/`. Se puede volver a ejecutar para completar una instalación interrumpida.

## Compilar

```powershell
.\build.ps1
```

La aplicación de escritorio y la herramienta de consola se crean en `target\release`. El script coloca `yt-dlp.exe` y `deno.exe` junto a ellas; distribuye los tres ejecutables juntos.

Abre la aplicación haciendo doble clic en:

```text
target\release\texto-youtube.exe
```

La interfaz permite pegar el enlace, elegir los idiomas (por defecto `es,en`), obtener la transcripción y copiarla al portapapeles. Puedes escribir varios códigos separados por comas, como `fr,ja`; no se solicitan todos los idiomas para reducir las peticiones a YouTube. Para compilar en modo de desarrollo:

```powershell
.\build.ps1 -DebugBuild
```

## Usar la consola

```powershell
.\target\release\texto-youtube-cli.exe "https://www.youtube.com/watch?v=ID_DEL_VIDEO" es,en
```

También se puede compilar y ejecutar en un paso:

```powershell
.\build.ps1 -Run "https://www.youtube.com/watch?v=ID_DEL_VIDEO"
```

El programa busca subtítulos publicados o automáticos. Si el video no ofrece subtítulos, no puede generar una transcripción. Se usa el primer archivo de subtítulos que `yt-dlp` entrega; el texto se imprime sin tiempos ni etiquetas de formato. La disponibilidad y las condiciones de acceso dependen de YouTube y del video.

## Solución de problemas

- Si falta `deno.exe` o `yt-dlp.exe`, vuelve a ejecutar `.\setup.ps1` y después `.\build.ps1`.
- Si la configuración no puede descargar las herramientas, comprueba que haya conexión a Internet y vuelve a ejecutar `.\setup.ps1`. Los archivos se guardan dentro de `.tools/`.
- Si aparece HTTP 429, YouTube limitó temporalmente las solicitudes. Espera unos minutos antes de reintentar y solicita solo los idiomas necesarios.
- Si `yt-dlp` informa que el video no ofrece subtítulos, prueba otro video que tenga subtítulos disponibles; el programa no genera transcripciones por sí mismo.

## Archivos locales

- `.tools/`: Rust, `yt-dlp`, Deno y sus archivos locales.
- `target/`: ejecutables y archivos de compilación de Cargo.

Ambas carpetas están excluidas de Git. `Cargo.lock` se conserva para registrar las versiones exactas de las dependencias Rust.
