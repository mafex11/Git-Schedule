# git-schedule Windows installer
# Usage: irm https://raw.githubusercontent.com/mafex11/git-schedule/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "mafex11/git-schedule"
$installDir = "$env:LOCALAPPDATA\git-schedule"

Write-Host "Installing git-schedule..." -ForegroundColor Cyan

# Get latest release
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
$asset = $release.assets | Where-Object { $_.name -like "*windows*" } | Select-Object -First 1

if (-not $asset) {
    Write-Host "Error: Could not find Windows release" -ForegroundColor Red
    exit 1
}

$zipUrl = $asset.browser_download_url
$zipFile = "$env:TEMP\git-schedule.zip"

Write-Host "Downloading from $zipUrl..."
Invoke-WebRequest -Uri $zipUrl -OutFile $zipFile

# Create install directory
if (Test-Path $installDir) {
    Remove-Item -Recurse -Force $installDir
}
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Extract
Write-Host "Extracting to $installDir..."
Expand-Archive -Path $zipFile -DestinationPath $installDir -Force
Remove-Item $zipFile

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
}

Write-Host ""
Write-Host "git-schedule installed successfully!" -ForegroundColor Green
Write-Host "Restart your terminal, then run: git-schedule --help"
