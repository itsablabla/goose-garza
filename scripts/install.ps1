# Garza Goose one-shot installer for Windows (PowerShell)
# Usage:  iwr -UseB https://downloads.garzaos.cloud/install.ps1 | iex
$ErrorActionPreference = "Stop"

$GooseVersion = "1.31.0"
$ServerUrl    = "https://goosed.garzaos.cloud"
$ServerSecret = "garza-goosed-secret-2c0a4f7b9e3d81a4c6b2e8f5"

$releaseBase = "https://github.com/block/goose/releases/download/v$GooseVersion"
$zipUrl      = "$releaseBase/Goose-win32-x64.zip"
$installDir  = "$env:LOCALAPPDATA\Programs\Goose"
$settingsDir = "$env:APPDATA\Goose"
$tmpZip      = "$env:TEMP\GarzaGoose.zip"

Write-Host "==> Downloading Goose $GooseVersion for Windows..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing

Write-Host "==> Extracting to $installDir..." -ForegroundColor Cyan
if (Test-Path $installDir) { Remove-Item $installDir -Recurse -Force }
New-Item -ItemType Directory -Path $installDir | Out-Null
Expand-Archive -Path $tmpZip -DestinationPath $installDir -Force
Remove-Item $tmpZip

Write-Host "==> Writing pre-filled settings.json..." -ForegroundColor Cyan
if (-not (Test-Path $settingsDir)) { New-Item -ItemType Directory -Path $settingsDir | Out-Null }
$settingsFile = Join-Path $settingsDir "settings.json"
if (Test-Path $settingsFile) {
  $existing = Get-Content $settingsFile -Raw | ConvertFrom-Json
} else {
  $existing = New-Object PSObject
}
$external = @{
  enabled = $true
  url     = $ServerUrl
  secret  = $ServerSecret
}
$existing | Add-Member -NotePropertyName externalGoosed -NotePropertyValue $external -Force
$existing | ConvertTo-Json -Depth 10 | Set-Content $settingsFile -Encoding UTF8

$shortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Garza Goose.lnk"
$shell = New-Object -ComObject WScript.Shell
$link = $shell.CreateShortcut($shortcut)
$link.TargetPath = Join-Path $installDir "Goose.exe"
$link.Save()

Write-Host "==> Done! Launching Goose..." -ForegroundColor Green
Start-Process (Join-Path $installDir "Goose.exe")

Write-Host "Garza Goose pre-configured:"
Write-Host "  Server: $ServerUrl"
Write-Host "  Secret: (hidden)"
Write-Host "Enjoy!"