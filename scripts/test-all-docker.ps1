<#
Run tests inside all Docker images (base, Debian, Fedora).

Usage:
  .\scripts\test-all-docker.ps1 [-IncludeIgnored]

If -IncludeIgnored is provided, the script will run `cargo test -- --ignored` in
each container in addition to the regular tests.
#>

param(
  [switch]$IncludeIgnored
)

function Test-DockerAvailable {
  try {
    docker info > $null 2>&1
    return $true
  } catch {
    return $false
  }
}

if (-not (Test-DockerAvailable)) {
  Write-Error "Docker daemon not available. Start Docker Desktop or ensure Docker engine is running."
  Write-Host "Troubleshooting steps:"
  Write-Host "  - On Windows: start Docker Desktop and ensure 'Use the WSL 2 based engine' (or equivalent) is enabled."
  Write-Host "  - If running from WSL: enable WSL integration for your distro in Docker Desktop (Settings → Resources → WSL Integration)."
  Write-Host "  - Verify Docker is responding: run 'docker info' in the same shell you plan to use."
  Write-Host "More info: https://docs.docker.com/desktop/windows/wsl/"
  exit 1
}

$images = @(
  @{ name = 'check_vpn_tests'; file = 'contrib/Dockerfile' },
  @{ name = 'check_vpn_tests_debian'; file = 'contrib/Dockerfile.debian' },
  @{ name = 'check_vpn_tests_fedora'; file = 'contrib/Dockerfile.fedora' }
)

foreach ($img in $images) {
  $iname = $img.name
  $df = $img.file
  Write-Host "\n=== Building image: $iname (Dockerfile: $df) ==="
  docker build -t $iname -f $df .
  if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build image $iname"
    exit $LASTEXITCODE
  }

  Write-Host "=== Running tests in $iname ==="
  docker run --rm $iname sh -c "cargo test"
  if ($LASTEXITCODE -ne 0) {
    Write-Error "Tests failed in $iname"
    exit $LASTEXITCODE
  }

  if ($IncludeIgnored) {
    Write-Host "=== Running ignored tests in $iname ==="
    docker run --rm $iname sh -c "cargo test -- --ignored"
    if ($LASTEXITCODE -ne 0) {
      Write-Error "Ignored tests failed in $iname"
      exit $LASTEXITCODE
    }
  }
}

Write-Host "All Docker test runs completed successfully."
