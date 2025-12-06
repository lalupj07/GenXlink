# GenXLink Distribution Build Script
# Run this script to build all installer formats

param(
    [switch]$All,
    [switch]$Portable,
    [switch]$NSIS,
    [switch]$Inno,
    [switch]$MSI
)

$ErrorActionPreference = "Stop"
$Version = "1.0.0"

Write-Host "GenXLink Distribution Builder v$Version" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan

# Build release if needed
if (-not (Test-Path "target\release\genxlink.exe")) {
    Write-Host "`nBuilding release..." -ForegroundColor Yellow
    cargo build --release --manifest-path client\ui-demo\Cargo.toml
}

# Create dist folders
New-Item -ItemType Directory -Force -Path "dist\windows-portable" | Out-Null
New-Item -ItemType Directory -Force -Path "dist\windows-installer" | Out-Null

# Copy executable
Copy-Item "target\release\genxlink.exe" "dist\windows-portable\GenXLink.exe" -Force

# 1. Portable ZIP
if ($All -or $Portable -or (-not $NSIS -and -not $Inno -and -not $MSI)) {
    Write-Host "`n[1/4] Creating Portable ZIP..." -ForegroundColor Green
    Compress-Archive -Path "dist\windows-portable\*" -DestinationPath "dist\GenXLink-v$Version-Windows-Portable.zip" -Force
    Write-Host "  Created: GenXLink-v$Version-Windows-Portable.zip"
}

# 2. NSIS Installer
if ($All -or $NSIS) {
    Write-Host "`n[2/4] Building NSIS Installer..." -ForegroundColor Green
    $nsis = Get-Command makensis -ErrorAction SilentlyContinue
    if ($nsis) {
        Push-Location "dist\windows-installer"
        makensis GenXLink.nsi
        Pop-Location
        Write-Host "  Created: GenXLink-v$Version-Setup.exe (NSIS)"
    } else {
        Write-Host "  NSIS not installed - skipping" -ForegroundColor Yellow
    }
}

# 3. Inno Setup Installer
if ($All -or $Inno) {
    Write-Host "`n[3/4] Building Inno Setup Installer..." -ForegroundColor Green
    $inno = Get-Command iscc -ErrorAction SilentlyContinue
    if ($inno) {
        iscc "dist\windows-installer\GenXLink.iss"
        Write-Host "  Created: GenXLink-v$Version-Setup.exe (Inno)"
    } else {
        Write-Host "  Inno Setup not installed - skipping" -ForegroundColor Yellow
    }
}

# 4. WiX MSI
if ($All -or $MSI) {
    Write-Host "`n[4/4] Building WiX MSI..." -ForegroundColor Green
    $wix = Get-Command candle -ErrorAction SilentlyContinue
    if ($wix) {
        Push-Location "dist\windows-installer"
        candle GenXLink.wxs -out GenXLink.wixobj
        light GenXLink.wixobj -out "GenXLink-v$Version.msi" -ext WixUIExtension
        Pop-Location
        Write-Host "  Created: GenXLink-v$Version.msi"
    } else {
        Write-Host "  WiX Toolset not installed - skipping" -ForegroundColor Yellow
    }
}

Write-Host "`n======================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "`nOutput files in: dist\" -ForegroundColor Cyan
Get-ChildItem "dist\*.zip", "dist\*.exe", "dist\*.msi" -ErrorAction SilentlyContinue | ForEach-Object {
    $size = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  $($_.Name) ($size MB)"
}
