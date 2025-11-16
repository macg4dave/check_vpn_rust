param(
  [switch]$Release
)

Write-Host "Starting Windows build (Release: $Release)"

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
  Write-Error "rustc not found. Install rustup from https://rustup.rs"
  exit 1
}

# Recommend MSVC toolchain on Windows: `rustup default stable-x86_64-pc-windows-msvc`
if ($Release) {
  cargo build --release
} else {
  cargo build
}

# Package the produced binary into a zip for CI/consumption
if ($Release) {
  $bin = Join-Path -Path "target" -ChildPath "release"
} else {
  $bin = Join-Path -Path "target" -ChildPath "debug"
}
$exe = Join-Path $bin 'check_vpn.exe'
if (-not (Test-Path $exe)) {
  # Try without extension (if using GNU toolchain)
  $exe = Join-Path $bin 'check_vpn'
}

if (Test-Path $exe) {
  $out = Join-Path $bin 'checkvpn-windows.zip'
  if (Test-Path $out) { Remove-Item $out }
  Compress-Archive -Path $exe -DestinationPath $out
  Write-Host "Packaged: $out"
} else {
  Write-Warning "Built binary not found; skipping packaging"
}
