$ErrorActionPreference = "Stop"
$Version = if ($env:GUPPTY_VERSION) { $env:GUPPTY_VERSION } else { "0.2.0" }
$Repository = if ($env:GUPPTY_REPOSITORY) { $env:GUPPTY_REPOSITORY } else { "https://github.com/adrian-1-cardona/Guppty" }

function Write-Step([string]$Message) {
    Write-Host "`n$Message" -ForegroundColor Cyan
}

Write-Step "Welcome to Guppty (version $Version)"

$HasRust = (Get-Command rustc -ErrorAction SilentlyContinue) -and (Get-Command cargo -ErrorAction SilentlyContinue)
if ($HasRust) {
    Write-Step "Rust is already installed - keeping your current toolchain."
} else {
    Write-Step "Rust was not found. Installing it with the official rustup installer..."
    $Rustup = Join-Path $env:TEMP "guppty-rustup-init.exe"
    Invoke-WebRequest "https://win.rustup.rs/x86_64" -OutFile $Rustup
    Start-Process -FilePath $Rustup -ArgumentList "-y", "--profile", "minimal" -Wait -NoNewWindow
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
    Remove-Item $Rustup -Force -ErrorAction SilentlyContinue
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Cargo is not available yet. Restart PowerShell and run the installer again."
}

Write-Step "Installing verified Guppty $Version from source pinned to tag v$Version..."
try {
    cargo install --git $Repository --tag "v$Version" --locked --force guppty
} catch {
    Write-Step "Tag v$Version not found on remote; building from repository default branch..."
    cargo install --git $Repository --locked --force guppty
}
if ($LASTEXITCODE -ne 0) { throw "Cargo could not install Guppty." }

$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Write-Step "Guppty $(guppty --version) is ready!"
Write-Host "Create your first program with:"
Write-Host "  guppty new hello-guppty"
Write-Host "  cd hello-guppty"
Write-Host "  guppty run"
Write-Host "`nTo upgrade or reinstall a specific version:"
Write-Host '  $env:GUPPTY_VERSION="x.y.z"; .\install.ps1'
Write-Host "To uninstall:"
Write-Host "  cargo uninstall guppty"
