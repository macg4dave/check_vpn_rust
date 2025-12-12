$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

param(
    [string]$Version
)

$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location $repoRoot
try {
    # Build release binary
    cargo build --release

    $artifactDir = Join-Path $repoRoot 'artifacts\windows'
    New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

    $bin = Join-Path $repoRoot 'target\release\check_vpn.exe'
    if (-not (Test-Path $bin)) {
        # Fallback name if cargo did not append .exe for some reason
        $bin = Join-Path $repoRoot 'target\release\check_vpn'
    }
    Copy-Item $bin (Join-Path $artifactDir 'check_vpn.exe') -Force

    # Include sample config and README for convenience
    Copy-Item (Join-Path $repoRoot 'contrib\config.xml') (Join-Path $artifactDir 'config.xml') -Force
    Copy-Item (Join-Path $repoRoot 'README.md') (Join-Path $artifactDir 'README.md') -Force

    $zipName = if ($Version) { "check_vpn-windows-$Version.zip" } else { "check_vpn-windows.zip" }
    $zipPath = Join-Path $artifactDir $zipName
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }

    Compress-Archive -Path (Join-Path $artifactDir '*') -DestinationPath $zipPath -Force
    Write-Host "Packaged Windows build at $zipPath"
}
finally {
    Pop-Location
}
