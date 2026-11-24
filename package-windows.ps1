param(
    [string]$BinaryDirectory = (Join-Path $PSScriptRoot 'target\release'),
    [string]$YtDlpPath = (Join-Path $PSScriptRoot '.tools\yt-dlp.exe'),
    [string]$DenoPath = (Join-Path $PSScriptRoot '.tools\deno\deno.exe'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'dist')
)

$ErrorActionPreference = 'Stop'
$cargoToml = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'Cargo.toml') -Raw
$versionMatch = [regex]::Match($cargoToml, '(?ms)^\[package\].*?^version\s*=\s*"(\d+\.\d+\.\d+)"')
if (-not $versionMatch.Success) { throw 'No pude obtener una versión MSI válida desde Cargo.toml.' }
$productVersion = $versionMatch.Groups[1].Value
$packageName = "texto-youtube-windows-x64-$productVersion.zip"
$stablePackageName = 'texto-youtube-windows-x64.zip'
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

    Copy-Item -LiteralPath $zipPath -Destination (Join-Path $OutputDirectory $stablePackageName) -Force
    $msiPath = Join-Path $OutputDirectory "texto-youtube-windows-x64-$productVersion.msi"
    $wixSource = Join-Path $PSScriptRoot 'installer\Product.wxs'
    if (-not (Test-Path -LiteralPath $wixSource -PathType Leaf)) { throw "Falta la definición del instalador: $wixSource" }
    & dotnet tool run wix -- build $wixSource -arch x64 -d "ProductVersion=$productVersion" -d "BinaryDirectory=$stagingDirectory" -o $msiPath
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $msiPath -PathType Leaf)) { throw 'No se pudo compilar el instalador MSI.' }
    # wix build runs the stock MSI ICE validation as part of package creation.
    $parts = $productVersion.Split('.') | ForEach-Object { [int]$_ }
    if ($parts[2] -gt 0) { $parts[2]-- }
    elseif ($parts[1] -gt 0) { $parts[1]--; $parts[2] = 255 }
    elseif ($parts[0] -gt 0) { $parts[0]--; $parts[1] = 255; $parts[2] = 255 }
    else { throw 'La versión 0.0.0 no permite probar una actualización mayor.' }
    $previousVersion = $parts -join '.'
    $upgradeFixture = Join-Path $OutputDirectory 'upgrade-test.msi'
    & dotnet tool run wix -- build $wixSource -arch x64 -d "ProductVersion=$previousVersion" -d "BinaryDirectory=$stagingDirectory" -o $upgradeFixture
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $upgradeFixture -PathType Leaf)) { throw 'No se pudo compilar el MSI previo para probar la actualización.' }
    Write-Host "Paquete portátil listo: $zipPath"
    Write-Host "Instalador MSI listo: $msiPath"
}
finally {
    $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\') + '\'
    $stagingPath = [System.IO.Path]::GetFullPath($stagingDirectory)
    if ($stagingPath.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $stagingPath).StartsWith('texto-youtube-package-')) {
        Remove-Item -LiteralPath $stagingPath -Recurse -Force
    }
}
