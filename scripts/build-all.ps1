<#
Build-all helper for Windows users without GNU make.

This script builds the host release, Debian and Fedora Docker-based releases
and packages a Windows artifact. It will attempt a macOS build only when run
on macOS.
#>

param(
  [switch]$SkipDocker
)

Write-Host "build-all: Starting (SkipDocker=$SkipDocker)"


function Invoke-CargoRelease {
  Write-Host "Running cargo build --release"
  cargo build --release
}

function Invoke-DebianDockerBuild {
  if ($SkipDocker) { Write-Host "Skipping Debian Docker build (SkipDocker)"; return }
  if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Warning "Docker not found; skipping Debian Docker build"
    return
  }
  Write-Host "Building Debian Docker image"
  docker build -t check_vpn_debian -f contrib/Dockerfile.debian .
  Write-Host "Running Debian Docker build (cargo build --release inside container)"
  docker run --rm -v ${PWD}:/work -w /work check_vpn_debian `
    bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-debian.tar.gz -C target/release check_vpn* || true"
}

function Invoke-FedoraDockerBuild {
  if ($SkipDocker) { Write-Host "Skipping Fedora Docker build (SkipDocker)"; return }
  if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Warning "Docker not found; skipping Fedora Docker build"
    return
  }
  Write-Host "Building Fedora Docker image"
  docker build -t check_vpn_fedora -f contrib/Dockerfile.fedora .
  Write-Host "Running Fedora Docker build (cargo build --release inside container)"
  docker run --rm -v ${PWD}:/work -w /work check_vpn_fedora `
    bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-fedora.tar.gz -C target/release check_vpn* || true"
}

function Invoke-MacIfPresent {
  $is_osx = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)
  if ($is_osx) {
    Write-Host "Detected macOS: running macOS build"
    cargo build --release
  } else {
    Write-Host "Skipping macOS build: not running on macOS"
  }
}

function Invoke-WindowsIfPresent {
  $is_windows = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
  if ($is_windows) {
    Write-Host "Detected Windows: running Windows build script"
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-windows.ps1 -Release
  } else {
    Write-Host "Skipping Windows build: not running on Windows"
  }
}

# Execute steps
Invoke-CargoRelease
Invoke-DebianDockerBuild
Invoke-FedoraDockerBuild
Invoke-MacIfPresent
Invoke-WindowsIfPresent

Write-Host "build-all: finished"
