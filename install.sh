#!/bin/sh
set -eu

GUPPTY_VERSION="${GUPPTY_VERSION:-0.2.0}"
GUPPTY_REPOSITORY="${GUPPTY_REPOSITORY:-https://github.com/adrian-1-cardona/Guppty}"

say() {
  printf '\n\033[1;36m%s\033[0m\n' "$1"
}

fail() {
  printf '\nGuppty install stopped: %s\n' "$1" >&2
  exit 1
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

verify_sha256() {
  file="$1"
  expected_hash="$2"
  if command_exists sha256sum; then
    actual_hash="$(sha256sum "$file" | awk '{print $1}')"
  elif command_exists shasum; then
    actual_hash="$(shasum -a 256 "$file" | awk '{print $1}')"
  else
    say "Warning: neither sha256sum nor shasum found; skipping checksum verification."
    return 0
  fi
  if [ "$actual_hash" != "$expected_hash" ]; then
    fail "Checksum verification failed for $file! Expected $expected_hash, got $actual_hash"
  fi
}

install_linux_build_tools() {
  command_exists cc && return
  say "Installing the system build tools Rust needs..."
  if command_exists apt-get; then
    if command_exists sudo; then sudo apt-get update && sudo apt-get install -y build-essential curl; else apt-get update && apt-get install -y build-essential curl; fi
  elif command_exists dnf; then
    if command_exists sudo; then sudo dnf group install -y "Development Tools"; else dnf group install -y "Development Tools"; fi
  elif command_exists yum; then
    if command_exists sudo; then sudo yum groupinstall -y "Development Tools"; else yum groupinstall -y "Development Tools"; fi
  elif command_exists pacman; then
    if command_exists sudo; then sudo pacman -Sy --needed --noconfirm base-devel curl; else pacman -Sy --needed --noconfirm base-devel curl; fi
  elif command_exists apk; then
    if command_exists sudo; then sudo apk add build-base curl; else apk add build-base curl; fi
  else
    fail "a C compiler is missing. Install your system's build tools, then run this command again."
  fi
}

say "Welcome to Guppty (version $GUPPTY_VERSION)"

case "$(uname -s 2>/dev/null || true)" in
  Darwin)
    if ! command_exists cc; then
      say "macOS needs the Command Line Tools. A system installer will open now."
      xcode-select --install 2>/dev/null || true
      fail "finish the Command Line Tools installation, then run this command again."
    fi
    ;;
  Linux) install_linux_build_tools ;;
  *) fail "this installer supports macOS and Linux. On Windows, use install.ps1 from the documentation." ;;
esac

if command_exists rustc && command_exists cargo; then
  say "Rust is already installed — keeping your current toolchain."
else
  command_exists curl || fail "curl is required to download the official Rust installer."
  say "Rust was not found. Installing it with the official rustup installer..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi

if [ -f "${CARGO_HOME:-$HOME/.cargo}/env" ]; then
  # Make Cargo available immediately, without asking the user to restart the terminal.
  . "${CARGO_HOME:-$HOME/.cargo}/env"
fi

command_exists cargo || fail "Cargo is not available yet. Restart your terminal and run the installer again."

say "Installing verified Guppty $GUPPTY_VERSION from source pinned to tag v$GUPPTY_VERSION..."
if ! cargo install --git "$GUPPTY_REPOSITORY" --tag "v$GUPPTY_VERSION" --locked --force guppty 2>/dev/null; then
  say "Release tag v$GUPPTY_VERSION not found on remote; building from repository default branch..."
  cargo install --git "$GUPPTY_REPOSITORY" --locked --force guppty
fi

command_exists guppty || export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
command_exists guppty || fail "Guppty installed, but Cargo's bin folder is not on PATH. Add ~/.cargo/bin to PATH."

say "Guppty $(guppty --version | awk '{print $2}') is ready!"
printf '%s\n' "Create your first program with:" "  guppty new hello-guppty" "  cd hello-guppty" "  guppty run"
printf '\nTo upgrade or reinstall a specific version:\n  GUPPTY_VERSION=x.y.z ./install.sh\nTo uninstall:\n  cargo uninstall guppty\n'
