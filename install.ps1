# OneSource Network Installer
# Run with: irm https://raw.githubusercontent.com/TW-RF54732/OneSource/main/install.ps1 | iex

$Repo = "TW-RF54732/OneSource"
$InstallDir = "$env:LOCALAPPDATA\Programs\OneSource"
$ExeName = "OneSource.exe"

# Clear screen for a fresh look
Clear-Host

# ----------------------------------------------------------
# BANNER (Static, no dynamic version needed here)
# ----------------------------------------------------------
Write-Host "
==========================================================
  ____  _   _ _____   ____   ___  _   _ ____   ____ _____ 
 / __ \| \ | | ____| / ___| / _ \| | | |  _ \ / ___| ____|
| |  | |  \| |  _|   \___ \| | | | | | | |_) | |   |  _|  
| |__| | |\  | |___   ___) | |_| | |_| |  _ <| |___| |___ 
 \____/|_| \_|_____| |____/ \___/ \___/|_| \_\\____|_____|
                          
 >> OneSource Network Installer | Vibe Coding Edition <<
==========================================================
" -ForegroundColor Cyan

# 1. Get Latest Release Info
Write-Host "[1/4] Fetching latest release info from GitHub..."
try {
    # Fetch release data from GitHub API
    $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    
    # Find the executable asset
    $Asset = $Release.assets | Where-Object { $_.name -eq $ExeName } | Select-Object -First 1
    
    if (-not $Asset) { throw "Could not find '$ExeName' in the latest release." }
    
    $DownloadUrl = $Asset.browser_download_url
    $Version = $Release.tag_name
    
    Write-Host "      Found Version: $Version" -ForegroundColor Gray
}
catch {
    Write-Error "Failed to fetch release info. Please check your internet connection."
    Write-Error "Error details: $_"
    exit 1
}

# 2. Setup Directory
Write-Host "[2/4] Preparing installation directory..."
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Write-Host "      Created: $InstallDir" -ForegroundColor Gray
} else {
    Write-Host "      Directory exists." -ForegroundColor Gray
}

# 3. Download
Write-Host "[3/4] Downloading $ExeName ($Version)..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile "$InstallDir\$ExeName"
    Write-Host "      Download complete." -ForegroundColor Gray
}
catch {
    Write-Error "Download failed: $_"
    exit 1
}

# 4. Update PATH
Write-Host "[4/4] Configuring environment (PATH)..."
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "User")
    Write-Host "      [SUCCESS] Added OneSource to your User PATH." -ForegroundColor Green
} else {
    Write-Host "      [SKIP] OneSource is already in your PATH." -ForegroundColor Gray
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host "  INSTALLATION COMPLETE!" -ForegroundColor Green
Write-Host "  Location: $InstallDir\$ExeName"
Write-Host "  Version:  $Version"
Write-Host ""
Write-Host "  * IMPORTANT: You may need to RESTART your terminal *"
Write-Host "    to use the 'OneSource' command."
Write-Host "==========================================================" -ForegroundColor Cyan