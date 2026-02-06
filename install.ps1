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
    Write-Host "Adding to user PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
}

# Also add to system PATH so it's picked up by all sessions
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($machinePath -notlike "*$installDir*") {
    try {
        Write-Host "Adding to system PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$machinePath;$installDir", "Machine")
    } catch {
        Write-Host "Note: Could not add to system PATH (requires admin). Run as Administrator if needed." -ForegroundColor Yellow
    }
}

# Update current session PATH so it works immediately
if ($env:Path -notlike "*$installDir*") {
    $env:Path = "$env:Path;$installDir"
}

Write-Host ""
Write-Host @"
⣿⣿⣿⣿⣿⣷⣿⣿⣿⡅⡹⢿⠆⠙⠋⠉⠻⠿⣿⣿⣿⣿⣿⣿⣮⠻⣦⡙⢷⡑⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣌⠡⠌⠂⣙⠻⣛⠻⠷⠐⠈⠛⢱⣮⣷⣽⣿
⣿⣿⣿⣿⡇⢿⢹⣿⣶⠐⠁⠀⣀⣠⣤⠄⠀⠀⠈⠙⠻⣿⣿⣿⣦⣵⣌⠻⣷⢝⠦⠚⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢟⣻⣿⣊⡃⠀⣙⠿⣿⣿⣿⣎⢮⡀⢮⣽⣿⣿
⢿⣿⣿⣿⣧⡸⡎⡛⡩⠖⠀⣴⣿⣿⣿⠀⠀⠀⠀⠸⠇⠀⠙⢿⣿⣿⣿⣷⣌⢷⣑⢷⣄⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⣫⠶⠛⠉⠀⠁⠀⠈⠈⠀⠠⠜⠻⣿⣆⢿⣼⣿⣿⣿
⢐⣿⣿⣿⣿⣧⢧⣧⢻⣦⢀⣹⣿⣿⣿⣇⠀⠄⠀⠀⠀⡀⠀⠈⢻⣿⣿⣿⣿⣷⣝⢦⡹⠷⡙⢿⣿⣿⣿⣿⣿⣿⣿⣿⠈⠁⠀⠀⠀⠁⠀⠀⠀⠱⣶⣄⡀⠀⠈⠛⠜⣿⣿⣿⣿
⠀⠊⢫⣿⣏⣿⡌⣼⣄⢫⡌⣿⣿⣿⣿⣿⣦⡈⠲⣄⣤⣤⡡⢀⣠⣿⣿⣿⣿⣿⣿⣷⣼⣍⢬⣦⡙⣿⣿⣿⣿⣿⣯⢁⡄⠀⡀⡀⠀⠄⢈⣠⢪⠀⣿⣿⣿⣦⠀⢉⢂⠹⡿⣿⣿
⠀⠀⠄⢹⢃⢻⣟⠙⣿⣦⠱⢻⣿⣿⣿⣿⣿⣿⣷⣬⣍⣭⣥⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⡙⢿⣼⡿⣿⣿⣿⣿⣿⣷⣄⠘⣱⢦⣤⡴⡿⢈⣼⣿⣿⣿⣇⣴⣶⣮⣅⢻⣿⡏
⠀⠀⠈⠹⣇⢡⢿⡆⠻⣿⣷⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣍⡻⣿⣟⣻⣿⣿⣿⣿⣷⣦⣥⣬⣤⣴⣾⣿⣿⣿⣿⣷⣿⣿⣿⣿⣷⡜⠃
⠀⠀⠀⢀⣘⠈⢂⠃⣧⡹⣿⣷⡄⠙⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣮⣅⡙⢿⣟⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠋⡕⠂
⠀⠀⠀⠀⠀⠀⠛⢷⣜⢷⡌⠻⣿⣿⣦⣝⣻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣯⣹⣷⣦⣹⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠉⠃⠀

       git-schedule installed successfully!
       Schedule your commits like a pro~
"@ -ForegroundColor Magenta
Write-Host ""
Write-Host "  Quick start:" -ForegroundColor White
Write-Host "    git add myfile.rs" -ForegroundColor Gray
Write-Host "    git-schedule `"feat: my commit`" --in 2h" -ForegroundColor Yellow
Write-Host ""
Write-Host "  The daemon starts automatically when you schedule a commit." -ForegroundColor Gray
Write-Host ""
Write-Host "  Restart your terminal, then run: " -ForegroundColor White -NoNewline
Write-Host "git-schedule --help" -ForegroundColor Yellow
Write-Host ""
