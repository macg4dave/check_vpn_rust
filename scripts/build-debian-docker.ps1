Write-Host "Building Debian Docker image and running cargo build --release inside it"

# Build image
docker build -t check_vpn_debian -f contrib/Dockerfile.debian .

# Run build inside container, mount current dir
docker run --rm -v ${PWD}:/work -w /work check_vpn_debian `
  bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-debian.tar.gz -C target/release check_vpn* || true"

Write-Host "Debian build complete; artifact (if produced) will be in target/release/"
