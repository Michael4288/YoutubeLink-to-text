param(
    [Parameter(Mandatory = $true)][string]$PreviousMsi,
    [Parameter(Mandatory = $true)][string]$CurrentMsi
)

$ErrorActionPreference = 'Stop'
$installDirectory = Join-Path $env:LOCALAPPDATA 'Programs\TextoYoutube'
$startMenu = Join-Path ([Environment]::GetFolderPath('Programs')) 'Texto de YouTube\Texto de YouTube.lnk'
$preferences = Join-Path $env:APPDATA 'TextoYoutube\preferences.json'
$preferenceDirectory = Split-Path -Parent $preferences
$hadPreferences = Test-Path -LiteralPath $preferences
$previousPreferences = if ($hadPreferences) { [System.IO.File]::ReadAllBytes($preferences) } else { $null }

function Install-Msi([string]$Path) {
    $process = Start-Process msiexec.exe -ArgumentList @('/i', "`"$Path`"", '/qn', '/norestart') -Wait -PassThru -WindowStyle Hidden
    if ($process.ExitCode -notin @(0, 3010)) { throw "La instalación MSI falló con el código $($process.ExitCode)." }
}

try {
    Install-Msi $PreviousMsi
    Install-Msi $CurrentMsi
    foreach ($name in @('texto-youtube.exe', 'texto-youtube-cli.exe', 'yt-dlp.exe', 'deno.exe')) {
        if (-not (Test-Path -LiteralPath (Join-Path $installDirectory $name) -PathType Leaf)) { throw "Falta el ejecutable instalado: $name" }
    }
    if (-not (Test-Path -LiteralPath $startMenu -PathType Leaf)) { throw 'No se creó el acceso directo del menú Inicio.' }
    $uninstallEntries = @(Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall' | Get-ItemProperty | Where-Object DisplayName -eq 'Texto de YouTube')
    if ($uninstallEntries.Count -ne 1) { throw "Se esperaba una entrada de desinstalación, se encontraron $($uninstallEntries.Count)." }

    $cli = Start-Process -FilePath (Join-Path $installDirectory 'texto-youtube-cli.exe') -Wait -PassThru -WindowStyle Hidden
    if ($cli.ExitCode -ne 2) { throw "La CLI instalada no inició correctamente (código $($cli.ExitCode))." }
    $gui = Start-Process -FilePath (Join-Path $installDirectory 'texto-youtube.exe') -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    if ($gui.HasExited) { throw 'La GUI instalada se cerró inesperadamente.' }
    Stop-Process -Id $gui.Id -Force

    New-Item -ItemType Directory -Force -Path $preferenceDirectory | Out-Null
    [System.IO.File]::WriteAllText($preferences, '{"version":1,"languages":["sentinel"]}')
    $productCode = $uninstallEntries[0].PSChildName
    $uninstall = Start-Process msiexec.exe -ArgumentList @('/x', $productCode, '/qn', '/norestart') -Wait -PassThru -WindowStyle Hidden
    if ($uninstall.ExitCode -notin @(0, 3010)) { throw "La desinstalación MSI falló con el código $($uninstall.ExitCode)." }
    if (Test-Path -LiteralPath $installDirectory) { throw 'La desinstalación dejó archivos de la aplicación.' }
    if (Test-Path -LiteralPath $startMenu) { throw 'La desinstalación dejó el acceso directo.' }
    if ((Get-Content -LiteralPath $preferences -Raw) -ne '{"version":1,"languages":["sentinel"]}') { throw 'La desinstalación alteró las preferencias del usuario.' }
}
finally {
    if ($hadPreferences) { [System.IO.File]::WriteAllBytes($preferences, $previousPreferences) }
    elseif (Test-Path -LiteralPath $preferences) { Remove-Item -LiteralPath $preferences -Force }
}
