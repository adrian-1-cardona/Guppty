#!/usr/bin/env bash
# =============================================================================
# Guppty one-line installer
# =============================================================================
# Install with:
#   curl -fsSL https://raw.githubusercontent.com/adrian-1-cardona/guppty/main/install.sh | bash
#
# This script:
#   1. Checks for curl / git (or falls back to ZIP download)
#   2. Checks for Rust + Cargo — installs via rustup if missing
#   3. Builds and installs the `guppty` CLI onto your PATH
#   4. Verifies the install and prints next steps for writing .gup programs
# =============================================================================

set -euo pipefail

REPO_OWNER="adrian-1-cardona"
REPO_NAME="guppty"
REPO_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}.git"
ZIP_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/refs/heads/main.zip"
RAW_INSTALL_URL="https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/main/install.sh"

GUPPTY_HOME="${GUPPTY_HOME:-$HOME/.guppty}"
SRC_DIR="${GUPPTY_HOME}/src"
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"

# --- pretty printing ---------------------------------------------------------

if [[ -t 1 ]] && command -v tput >/dev/null 2>&1; then
  BOLD="$(tput bold 2>/dev/null || true)"
  DIM="$(tput dim 2>/dev/null || true)"
  GREEN="$(tput setaf 2 2>/dev/null || true)"
  YELLOW="$(tput setaf 3 2>/dev/null || true)"
  CYAN="$(tput setaf 6 2>/dev/null || true)"
  RED="$(tput setaf 1 2>/dev/null || true)"
  RESET="$(tput sgr0 2>/dev/null || true)"
else
  BOLD=""; DIM=""; GREEN=""; YELLOW=""; CYAN=""; RED=""; RESET=""
fi

info()  { printf '%s==>%s %s\n' "${CYAN}${BOLD}" "${RESET}" "$*"; }
ok()    { printf '%s✓%s  %s\n' "${GREEN}" "${RESET}" "$*"; }
warn()  { printf '%s!%s  %s\n' "${YELLOW}" "${RESET}" "$*"; }
fail()  { printf '%s✗%s  %s\n' "${RED}" "${RESET}" "$*" >&2; exit 1; }
step()  { printf '\n%s%s%s\n' "${BOLD}" "$*" "${RESET}"; }

banner() {
  cat <<EOF

${BOLD}${CYAN}  ╔═══════════════════════════════════════╗
  ║         Guppty fresh install          ║
  ║   write .gup  ·  compile  ·  run      ║
  ╚═══════════════════════════════════════╝${RESET}

EOF
}

# --- helpers -----------------------------------------------------------------

have() { command -v "$1" >/dev/null 2>&1; }

ensure_path_hint() {
  case ":${PATH}:" in
    *":${CARGO_BIN}:"*) return 0 ;;
  esac
  warn "${CARGO_BIN} is not on your PATH yet."
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
  export PATH="${CARGO_BIN}:${PATH}"
}

need_cmd() {
  if ! have "$1"; then
    fail "Missing required tool: $1. Please install it and re-run this script."
  fi
}

# --- steps -------------------------------------------------------------------

check_os() {
  step "1. Checking your system"
  local os
  os="$(uname -s 2>/dev/null || echo unknown)"
  case "$os" in
    Linux*|Darwin*) ok "Detected ${os}" ;;
    MINGW*|MSYS*|CYGWIN*)
      warn "Windows detected via ${os}."
      warn "Prefer: irm https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/main/install.ps1 | iex"
      ok "Continuing with bash install anyway"
      ;;
    *)
      warn "Unrecognized OS (${os}). Continuing — Rustup may still work."
      ;;
  esac
  need_cmd curl
  ok "curl is available"
}

ensure_rust() {
  step "2. Checking for Rust"
  ensure_path_hint

  if have rustc && have cargo; then
    ok "Rust already installed: $(rustc --version)"
    ok "Cargo already installed: $(cargo --version)"
    return 0
  fi

  warn "Rust not found — installing via rustup (non-interactive)"
  need_cmd curl

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
  export PATH="${CARGO_BIN}:${PATH}"

  if ! have rustc || ! have cargo; then
    fail "Rustup finished but rustc/cargo are still missing. Open a new terminal and re-run: curl -fsSL ${RAW_INSTALL_URL} | bash"
  fi

  ok "Installed $(rustc --version)"
  ok "Installed $(cargo --version)"
}

fetch_source() {
  step "3. Fetching a fresh Guppty source tree"
  mkdir -p "${GUPPTY_HOME}"
  rm -rf "${SRC_DIR}"
  mkdir -p "${SRC_DIR}"

  if have git; then
    info "Cloning ${REPO_URL}"
    git clone --depth 1 "${REPO_URL}" "${SRC_DIR}"
    ok "Cloned into ${SRC_DIR}"
  else
    warn "git not found — downloading source ZIP instead"
    local zip_path="${GUPPTY_HOME}/guppty-main.zip"
    curl -fsSL "${ZIP_URL}" -o "${zip_path}"
    if have unzip; then
      unzip -q "${zip_path}" -d "${GUPPTY_HOME}"
    elif have python3; then
      python3 - <<PY
import zipfile
zipfile.ZipFile("${zip_path}").extractall("${GUPPTY_HOME}")
PY
    else
      fail "Need unzip or python3 to extract the source ZIP (or install git)."
    fi
    # GitHub ZIP extracts to guppty-main/
    if [[ -d "${GUPPTY_HOME}/guppty-main" ]]; then
      mv "${GUPPTY_HOME}/guppty-main" "${SRC_DIR}"
    elif [[ -d "${GUPPTY_HOME}/Guppty-main" ]]; then
      mv "${GUPPTY_HOME}/Guppty-main" "${SRC_DIR}"
    else
      fail "Could not find extracted source directory."
    fi
    rm -f "${zip_path}"
    ok "Downloaded and extracted into ${SRC_DIR}"
  fi

  if [[ ! -f "${SRC_DIR}/Cargo.toml" ]]; then
    fail "Cargo.toml missing in ${SRC_DIR} — install may have fetched the wrong tree."
  fi
}

install_guppty() {
  step "4. Building and installing the guppty CLI"
  ensure_path_hint
  (
    cd "${SRC_DIR}"
    cargo install --path . --force
  )
  ok "guppty installed to ${CARGO_BIN}/guppty"
}

verify_install() {
  step "5. Verifying your install"
  ensure_path_hint

  if ! have guppty; then
    fail "guppty is not on PATH. Add ${CARGO_BIN} to PATH, then open a new terminal."
  fi

  ok "Found $(command -v guppty)"
  local demo="${SRC_DIR}/examples/hello.gup"
  if [[ -f "${demo}" ]]; then
    info "Running examples/hello.gup"
    guppty "${demo}"
    ok "Example ran successfully"
  else
    warn "Could not find examples/hello.gup — skipping demo run"
  fi
}

print_next_steps() {
  cat <<EOF

${GREEN}${BOLD}Guppty is ready.${RESET} You can write, compile, and run your own programs.

${BOLD}Create a new program${RESET}
  ${DIM}guppty new hello${RESET}
  ${DIM}# creates hello.gup in the current folder${RESET}

${BOLD}Compile it (checks for errors, no run)${RESET}
  ${DIM}guppty compile hello.gup${RESET}

${BOLD}Run it${RESET}
  ${DIM}guppty hello.gup${RESET}
  ${DIM}# or: guppty run hello.gup${RESET}

${BOLD}Help${RESET}
  ${DIM}guppty help${RESET}

Source checkout lives at: ${DIM}${SRC_DIR}${RESET}
Binary lives at:          ${DIM}${CARGO_BIN}/guppty${RESET}

If ${DIM}guppty${RESET} is not found in a new terminal, add this to your shell profile:
  ${DIM}export PATH="${CARGO_BIN}:\$PATH"${RESET}
  or: ${DIM}source "\$HOME/.cargo/env"${RESET}

Docs: ${CYAN}https://github.com/${REPO_OWNER}/${REPO_NAME}${RESET}

EOF
}

# --- main --------------------------------------------------------------------

main() {
  banner
  check_os
  ensure_rust
  fetch_source
  install_guppty
  verify_install
  print_next_steps
}

main "$@"
