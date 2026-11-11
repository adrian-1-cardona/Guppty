# Verified Immutable Releases and Safe Installation

This document defines Guppty's release process, binary artifact packaging, checksum verification, and safe installation paths.

## Semantic Versioning and Immutable Tags

Guppty releases follow Semantic Versioning (`MAJOR.MINOR.PATCH`).
Each release corresponds to an immutable Git tag (`vX.Y.Z`). Once pushed, release tags are never moved or overwritten.

## Binary Artifacts and Targets

Automated CI builds signed release archives for four Tier-1 platforms:
- Linux x86_64: `guppty-v{version}-x86_64-unknown-linux-gnu.tar.gz`
- macOS Apple Silicon (ARM64): `guppty-v{version}-aarch64-apple-darwin.tar.gz`
- macOS Intel (x86_64): `guppty-v{version}-x86_64-apple-darwin.tar.gz`
- Windows x86_64: `guppty-v{version}-x86_64-pc-windows-msvc.zip`

## Checksums and Provenance

Every release publishes:
1. `SHA256SUMS`: Cryptographic SHA-256 hashes of every distributed archive.
2. `provenance.json`: Metadata linking the release version, build timestamp, target triples, and exact source commit SHA.

To verify a downloaded artifact on macOS/Linux:
```bash
shasum -a 256 -c SHA256SUMS --ignore-missing
```

On Windows (PowerShell):
```powershell
Get-FileHash -Path <archive-file> -Algorithm SHA256
```

## Safe Installation and Version Pinning

### Default Install (Pins to current stable version `0.2.0`)
```bash
curl -fsSL https://raw.githubusercontent.com/adrian-1-cardona/Guppty/main/install.sh | sh
```

### Pinning an Explicit Release
To install an exact historical version for experiment reproducibility:
```bash
GUPPTY_VERSION=0.2.0 ./install.sh
```

On Windows PowerShell:
```powershell
$env:GUPPTY_VERSION="0.2.0"; .\install.ps1
```

### Upgrading Guppty
To upgrade to a newer release:
```bash
GUPPTY_VERSION=x.y.z ./install.sh
```

### Uninstalling Guppty
If installed via Cargo or the one-line installer:
```bash
cargo uninstall guppty
```
Or manually remove the binary from `~/.cargo/bin/guppty` (or `%USERPROFILE%\.cargo\bin\guppty.exe` on Windows).
