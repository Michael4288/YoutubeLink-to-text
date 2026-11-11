param(
    [switch]$DebugBuild,
    [string]$Run
)

$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$toolsDir = Join-Path $projectRoot '.tools'
$cargoHome = Join-Path $toolsDir 'cargo'
$rustupHome = Join-Path $toolsDir 'rustup'
$rustup = Join-Path $cargoHome 'bin\rustup.exe'
$gcc = Get-Command gcc -ErrorAction SilentlyContinue

if (-not (Test-Path (Join-Path $cargoHome 'bin\cargo.exe')) -or -not (Test-Path $rustup) -or -not (Test-Path (Join-Path $toolsDir 'yt-dlp.exe'))) {
    & (Join-Path $projectRoot 'setup.ps1')
    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not $gcc) {
    throw 'No se encontró gcc. Instala MinGW-w64 y asegúrate de que gcc esté en el PATH.'
}

$env:RUSTUP_HOME = $rustupHome
$env:CARGO_HOME = $cargoHome
$env:RUSTUP_DIST_SERVER = 'https://rsproxy.cn'
$env:RUSTUP_UPDATE_ROOT = 'https://rsproxy.cn/rustup'
$env:PATH = (Join-Path $cargoHome 'bin') + ';' + (Split-Path $gcc.Source -Parent) + ';' + $env:PATH

$installedToolchains = @(& $rustup toolchain list)
if (-not ($installedToolchains -match '^stable-x86_64-pc-windows-gnu')) {
    & (Join-Path $projectRoot 'setup.ps1')
    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$cargoArgs = @('build')
if (-not $DebugBuild) { $cargoArgs += '--release' }
$profile = if ($DebugBuild) { 'debug' } else { 'release' }
$profileDir = Join-Path $projectRoot "target\$profile"
Push-Location $projectRoot
try {
    & (Join-Path $cargoHome 'bin\cargo.exe') @cargoArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Copy-Item -LiteralPath (Join-Path $toolsDir 'yt-dlp.exe') -Destination (Join-Path $profileDir 'yt-dlp.exe') -Force

    if ($Run) {
        $binary = Join-Path $profileDir 'texto-youtube-cli.exe'
        & $binary $Run
        exit $LASTEXITCODE
    }
}
finally {
    Pop-Location
}
