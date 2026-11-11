$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$toolsDir = Join-Path $projectRoot '.tools'
$rustupHome = Join-Path $toolsDir 'rustup'
$cargoHome = Join-Path $toolsDir 'cargo'
$rustupInit = Join-Path $toolsDir 'rustup-init.exe'
$ytDlp = Join-Path $toolsDir 'yt-dlp.exe'

New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

if (-not (Test-Path $rustupInit)) {
    Write-Host 'Descargando instalador de Rust...'
    Invoke-WebRequest 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile $rustupInit
}

$env:RUSTUP_HOME = $rustupHome
$env:CARGO_HOME = $cargoHome
$env:RUSTUP_DIST_SERVER = 'https://rsproxy.cn'
$env:RUSTUP_UPDATE_ROOT = 'https://rsproxy.cn/rustup'
$env:RUSTUP_USE_CURL = '1'
$env:RUSTUP_DOWNLOAD_TIMEOUT = '300'
$rustup = Join-Path $cargoHome 'bin\rustup.exe'
if (-not (Test-Path $rustup)) {
    Write-Host 'Instalando Rust en .tools...'
    & $rustupInit -y --no-modify-path --profile minimal --default-host x86_64-pc-windows-gnu --default-toolchain stable
    if ($LASTEXITCODE -ne 0) { throw "La instalación local de Rust falló (código $LASTEXITCODE)." }
}

$installedToolchains = @(& $rustup toolchain list)
if (-not ($installedToolchains -match '^stable-x86_64-pc-windows-gnu')) {
    Write-Host 'Instalando el toolchain de Rust en .tools...'
    & $rustup toolchain install stable-x86_64-pc-windows-gnu --profile minimal
    if ($LASTEXITCODE -ne 0) { throw "La instalación local del toolchain falló (código $LASTEXITCODE)." }
}

if (-not (Test-Path $ytDlp)) {
    Write-Host 'Descargando yt-dlp en .tools...'
    Invoke-WebRequest 'https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe' -OutFile $ytDlp
}

Write-Host 'Herramientas locales listas en .tools.'
