Write-Host "Building Fedora Docker image and running cargo build --release inside it"

# Build image
docker build -t check_vpn_fedora -f contrib/Dockerfile.fedora .

# Run build inside container, mount current dir
docker run --rm -v ${PWD}:/work -w /work check_vpn_fedora `
  bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-fedora.tar.gz -C target/release check_vpn* || true"

Write-Host "Fedora build complete; artifact (if produced) will be in target/release/"
