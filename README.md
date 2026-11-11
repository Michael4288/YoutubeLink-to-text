# Texto de YouTube

Pequeña herramienta de línea de comandos escrita en Rust. Recibe un enlace de YouTube y muestra en la terminal la transcripción disponible, sin descargar el video.

## Requisitos

- Windows 10/11 de 64 bits y PowerShell.
- Conexión a Internet durante la configuración inicial.
- GCC para Windows (MinGW-w64) disponible como comando `gcc`. El proyecto incluye un entorno local de Rust; `yt-dlp` también se instala dentro del proyecto.

La configuración no modifica el `PATH` ni instala Rust o `yt-dlp` en el perfil del usuario. Ambos quedan bajo `.tools/`, que Git ignora. Rustup usa `rsproxy.cn` como espejo de descarga del toolchain.

## Configurar

Abre PowerShell en la carpeta del proyecto y ejecuta:

```powershell
.\setup.ps1
```

El script descarga Rust (toolchain GNU) y `yt-dlp` en `.tools/`. Se puede volver a ejecutar para completar una instalación interrumpida.

## Compilar

```powershell
.\build.ps1
```

El ejecutable optimizado se crea en `target\release\texto-youtube.exe`. Para compilar en modo de desarrollo:

```powershell
.\build.ps1 -DebugBuild
```

## Usar

```powershell
.\target\release\texto-youtube.exe "https://www.youtube.com/watch?v=ID_DEL_VIDEO"
```

También se puede compilar y ejecutar en un paso:

```powershell
.\build.ps1 -Run "https://www.youtube.com/watch?v=ID_DEL_VIDEO"
```

El programa busca subtítulos publicados o automáticos. Si el video no ofrece subtítulos, no puede generar una transcripción. Se usa el primer archivo de subtítulos que `yt-dlp` entrega; el texto se imprime sin tiempos ni etiquetas de formato. La disponibilidad y las condiciones de acceso dependen de YouTube y del video.

## Archivos locales

- `.tools/`: Rust, `yt-dlp` y sus archivos de configuración locales.
- `target/`: ejecutables y archivos de compilación de Cargo.

Ambas carpetas están excluidas de Git. `Cargo.lock` se conserva para registrar las versiones exactas de las dependencias Rust.
