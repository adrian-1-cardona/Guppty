# =============================================================================
# Guppty one-line installer (Windows PowerShell)
# =============================================================================
# Install with:
#   irm https://raw.githubusercontent.com/adrian-1-cardona/guppty/main/install.ps1 | iex
# =============================================================================

$ErrorActionPreference = "Stop"

$RepoOwner = "adrian-1-cardona"
$RepoName = "guppty"
$RepoUrl = "https://github.com/$RepoOwner/$RepoName.git"
$ZipUrl = "https://github.com/$RepoOwner/$RepoName/archive/refs/heads/main.zip"
$GupptyHome = if ($env:GUPPTY_HOME) { $env:GUPPTY_HOME } else { Join-Path $HOME ".guppty" }
$SrcDir = Join-Path $GupptyHome "src"
$CargoBin = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME "bin" } else { Join-Path $HOME ".cargo\bin" }

function Write-Info($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg) { Write-Host "OK  $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "!   $msg" -ForegroundColor Yellow }

Write-Host ""
Write-Host "  Guppty fresh install" -ForegroundColor Cyan
Write-Host "  write .gup  ·  compile  ·  run" -ForegroundColor DarkCyan
Write-Host ""

Write-Host "1. Checking for Rust" -ForegroundColor White
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
$cargo = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $rustc -or -not $cargo) {
  Write-Warn "Rust not found — installing via winget (Rustlang.Rustup)"
  winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
  $env:Path = "$CargoBin;" + [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
  $rustc = Get-Command rustc -ErrorAction SilentlyContinue
  $cargo = Get-Command cargo -ErrorAction SilentlyContinue
  if (-not $rustc -or -not $cargo) {
    throw "Rustup finished but rustc/cargo are still missing. Close this window, open a new PowerShell, and re-run the install command."
  }
} else {
  Write-Ok "Rust already installed: $(rustc --version)"
  Write-Ok "Cargo already installed: $(cargo --version)"
}

Write-Host ""
Write-Host "2. Fetching a fresh Guppty source tree" -ForegroundColor White
New-Item -ItemType Directory -Force -Path $GupptyHome | Out-Null
if (Test-Path $SrcDir) { Remove-Item -Recurse -Force $SrcDir }

$git = Get-Command git -ErrorAction SilentlyContinue
if ($git) {
  Write-Info "Cloning $RepoUrl"
  git clone --depth 1 $RepoUrl $SrcDir
} else {
  Write-Warn "git not found — downloading source ZIP instead"
  $zipPath = Join-Path $GupptyHome "guppty-main.zip"
  Invoke-WebRequest -Uri $ZipUrl -OutFile $zipPath
  Expand-Archive -Path $zipPath -DestinationPath $GupptyHome -Force
  $extracted = Join-Path $GupptyHome "guppty-main"
  if (-not (Test-Path $extracted)) {
    $extracted = Join-Path $GupptyHome "Guppty-main"
  }
  Move-Item $extracted $SrcDir
  Remove-Item $zipPath -Force
}
Write-Ok "Source ready at $SrcDir"

Write-Host ""
Write-Host "3. Building and installing the guppty CLI" -ForegroundColor White
Push-Location $SrcDir
try {
  cargo install --path . --force
} finally {
  Pop-Location
}
Write-Ok "guppty installed to $CargoBin\guppty.exe"

$env:Path = "$CargoBin;" + $env:Path
Write-Host ""
Write-Host "4. Verifying your install" -ForegroundColor White
$demo = Join-Path $SrcDir "examples\hello.gup"
if (Test-Path $demo) {
  & "$CargoBin\guppty.exe" $demo
  Write-Ok "Example ran successfully"
}

Write-Host ""
Write-Host "Guppty is ready." -ForegroundColor Green
Write-Host ""
Write-Host "Create a program:  guppty new hello"
Write-Host "Compile it:        guppty compile hello.gup"
Write-Host "Run it:            guppty hello.gup"
Write-Host "Help:              guppty help"
Write-Host ""
Write-Host "If guppty is not found in a new terminal, reopen PowerShell so PATH picks up $CargoBin"
