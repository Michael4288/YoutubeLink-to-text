param(
    [string]$BinaryDirectory = (Join-Path $PSScriptRoot 'target\release'),
    [string]$YtDlpPath = (Join-Path $PSScriptRoot '.tools\yt-dlp.exe'),
    [string]$DenoPath = (Join-Path $PSScriptRoot '.tools\deno\deno.exe'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'dist')
)

$ErrorActionPreference = 'Stop'
$packageName = 'texto-youtube-windows-x64.zip'
$requiredFiles = @('texto-youtube.exe', 'texto-youtube-cli.exe', 'yt-dlp.exe', 'deno.exe')
$sourcePaths = @{}

foreach ($file in $requiredFiles) {
    $source = switch ($file) {
        'texto-youtube.exe' { Join-Path $BinaryDirectory $file }
        'texto-youtube-cli.exe' { Join-Path $BinaryDirectory $file }
        'yt-dlp.exe' { $YtDlpPath }
        'deno.exe' { $DenoPath }
    }
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Falta el archivo requerido para el paquete: $source"
    }
    $resolvedSource = (Resolve-Path -LiteralPath $source).Path
    if ((Get-Item -LiteralPath $resolvedSource).Length -eq 0) {
        throw "El archivo requerido está vacío: $source"
    }
    $sourcePaths[$file] = $resolvedSource
}

New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$zipPath = Join-Path $OutputDirectory $packageName
$stagingDirectory = Join-Path ([System.IO.Path]::GetTempPath()) "texto-youtube-package-$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $stagingDirectory | Out-Null

try {
    foreach ($file in $requiredFiles) {
        Copy-Item -LiteralPath $sourcePaths[$file] -Destination (Join-Path $stagingDirectory $file)
    }

    Compress-Archive -Path (Join-Path $stagingDirectory '*') -DestinationPath $zipPath -CompressionLevel Optimal -Force

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $zipPath).Path)
    try {
        $packagedFiles = @($archive.Entries | ForEach-Object FullName | Sort-Object)
        $expectedFiles = @($requiredFiles | Sort-Object)
        $difference = Compare-Object -ReferenceObject $expectedFiles -DifferenceObject $packagedFiles
        if ($difference) {
            throw "El ZIP debe contener exactamente: $($expectedFiles -join ', '). Encontrado: $($packagedFiles -join ', ')"
        }
    }
    finally {
        $archive.Dispose()
    }

    Write-Host "Paquete listo: $zipPath"
}
finally {
    $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\') + '\'
    $stagingPath = [System.IO.Path]::GetFullPath($stagingDirectory)
    if ($stagingPath.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $stagingPath).StartsWith('texto-youtube-package-')) {
        Remove-Item -LiteralPath $stagingPath -Recurse -Force
    }
}
