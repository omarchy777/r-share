#!/usr/bin/env pwsh
# RShare Installer for Windows
# Run: irm https://raw.githubusercontent.com/ronakgh97/r-share/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

Write-Host "Installing RShare..." -ForegroundColor Cyan

# Configuration
$REPO = "ronakgh97/r-share"
$VERSION = "v1.0.0-beta"
$BINARY_NAME = "rs-win.exe"
$INSTALL_DIR = "$env:LOCALAPPDATA\rshare"
$INSTALL_NAME = "rs.exe"

# Download URL (from GitHub Release)
$DOWNLOAD_URL = "https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"

Write-Host "Downloading RShare $VERSION..." -ForegroundColor Yellow

# Create install directory
if (-not (Test-Path $INSTALL_DIR))
{
    New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
}

# Download binary
$BINARY_PATH = "$INSTALL_DIR\$INSTALL_NAME"
try
{
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $BINARY_PATH
    Write-Host "✓ Downloaded successfully" -ForegroundColor Green
}
catch
{
    Write-Host "✗ Download failed: $_" -ForegroundColor Red
    exit 1
}

# Add to PATH
Write-Host "Adding to PATH..." -ForegroundColor Yellow
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$INSTALL_DIR*")
{
    [Environment]::SetEnvironmentVariable(
            "Path",
            "$userPath;$INSTALL_DIR",
            "User"
    )
    $env:Path = "$env:Path;$INSTALL_DIR"
    Write-Host "✓ Added to PATH" -ForegroundColor Green
}
else
{
    Write-Host "✓ Already in PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host " ✓ Installation complete" -ForegroundColor Green
Write-Host ""