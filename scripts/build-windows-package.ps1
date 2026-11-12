param(
    [string]$Version = ""
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw "Release build failed ($LASTEXITCODE)" }

    $buildRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
    $bin = Join-Path $buildRoot 'release\check_vpn.exe'
    if (-not (Test-Path -LiteralPath $bin -PathType Leaf)) { throw "Release binary missing: $bin" }
    $binaryVersion = (& $bin --version) -replace '^check_vpn\s+', ''
    if ($LASTEXITCODE -ne 0) { throw 'Could not read binary version' }
    if (-not $Version) { $Version = $binaryVersion.Trim() }
    $Version = $Version -replace '^v', ''
    if ($Version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
        throw "Version must be a stable X.Y.Z version: $Version"
    }
    if ($Version -ne $binaryVersion.Trim()) { throw 'Package version must match the compiled binary' }

    $artifactDir = Join-Path $repoRoot 'artifacts\windows'
    New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
    $payload = @(
        (Join-Path $artifactDir 'check_vpn.exe'),
        (Join-Path $artifactDir 'config.xml'),
        (Join-Path $artifactDir 'README.md')
    )
    Copy-Item -LiteralPath $bin -Destination $payload[0] -Force
    Copy-Item -LiteralPath 'contrib\config.xml' -Destination $payload[1] -Force
    Copy-Item -LiteralPath 'README.md' -Destination $payload[2] -Force
    $zipPath = Join-Path $artifactDir "check_vpn-windows-$Version.zip"
    # An explicit payload prevents old ZIPs and unrelated artifacts being included.
    Compress-Archive -LiteralPath $payload -DestinationPath $zipPath -Force
    Write-Host "Packaged Windows build at $zipPath"
}
finally {
    Pop-Location
}
