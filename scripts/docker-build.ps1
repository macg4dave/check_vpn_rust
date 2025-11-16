<#
.SYNOPSIS
  Cross-platform Docker build helper for Windows PowerShell.

.DESCRIPTION
  Usage: .\docker-build.ps1 -Distro debian -Action build
  Distro: debian | fedora
  Action: build | build-release | test | test-ignored
#>

param(
  [ValidateSet('debian','fedora')]
  [string]$Distro = 'debian',

  [ValidateSet('build','build-release','test','test-ignored')]
  [string]$Action = 'build'
)

function Show-Usage {
  Write-Host "Usage: docker-build.ps1 -Distro <debian|fedora> -Action <build|build-release|test|test-ignored>"
}

switch ($Distro) {
  'debian' {
    $Dockerfile = 'contrib/Dockerfile.debian'
    $Image = 'check_vpn_tests_debian'
  }
  'fedora' {
    $Dockerfile = 'contrib/Dockerfile.fedora'
    $Image = 'check_vpn_tests_fedora'
  }
}

Write-Host "Building image $Image from $Dockerfile..."
docker build -t $Image -f $Dockerfile .

# Normalize host path for mounting into Docker. If running with Docker Desktop
# the Windows path usually works. Convert to POSIX-style /c/... path when
# appropriate (useful for some setups).
$hostPath = (Get-Location).ProviderPath

if ($hostPath -match '^[A-Za-z]:\\') {
  # Convert C:\Users\... to /c/Users/...
  $drive = $hostPath.Substring(0,1).ToLower()
  $rest = $hostPath.Substring(2) -replace '\\','/'
  $mountPath = "/$drive/$rest"
} else {
  $mountPath = $hostPath
}

$containerWD = '/usr/src/check_vpn'

switch ($Action) {
  'build' {
    docker run --rm -v "${mountPath}:$containerWD" -w $containerWD $Image sh -c "cargo build"
  }
  'build-release' {
    docker run --rm -v "${mountPath}:$containerWD" -w $containerWD $Image sh -c "cargo build --release"
  }
  'test' {
    docker run --rm $Image
  }
  'test-ignored' {
    docker run --rm $Image sh -c "cargo test -- --ignored"
  }
  default {
    Show-Usage
    exit 2
  }
}
