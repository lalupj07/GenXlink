# GenXLink Installer Build Script
# Copyright (c) 2025 GenXis Innovations
# Contact: genxisinnovation@outlook.com

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  GenXLink Installer Build Script" -ForegroundColor Cyan
Write-Host "  Version 0.1.0" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build Release Binary
Write-Host "[1/5] Building release binary..." -ForegroundColor Yellow
cargo build --release --package genxlink-windows
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Binary built successfully" -ForegroundColor Green
Write-Host ""

# Step 2: Create Distribution Directories
Write-Host "[2/5] Creating distribution directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "dist/portable" | Out-Null
New-Item -ItemType Directory -Force -Path "dist/installer" | Out-Null
Write-Host "✓ Directories created" -ForegroundColor Green
Write-Host ""

# Step 3: Create Portable Package
Write-Host "[3/5] Creating portable package..." -ForegroundColor Yellow
Copy-Item "target/release/genxlink.exe" "dist/portable/" -Force
Copy-Item "LICENSE" "dist/portable/" -Force
Copy-Item "COPYRIGHT" "dist/portable/" -Force
Copy-Item "README.md" "dist/portable/" -Force

# Create portable README
@"
================================================================================
                        GenXLink - Portable Edition
                     Remote Desktop Application v0.1.0
================================================================================

Copyright (c) 2025 GenXis Innovations
Licensed under Apache License 2.0
Contact: genxisinnovation@outlook.com

QUICK START:
1. Double-click genxlink.exe to run
2. No installation required
3. All settings stored in the same directory

FEATURES:
✓ Screen Capture & Streaming (30 FPS)
✓ Video Recording (MJPEG AVI)
✓ WebRTC Peer-to-Peer
✓ Remote Control (Mouse & Keyboard)
✓ Multi-Monitor Support

SUPPORT:
Email: genxisinnovation@outlook.com
GitHub: https://github.com/lalupj07/GenXlink

================================================================================
"@ | Out-File -FilePath "dist/portable/README.txt" -Encoding UTF8

# Create ZIP
Compress-Archive -Path "dist/portable/*" -DestinationPath "dist/GenXLink-v0.1.0-Portable-Windows-x64.zip" -Force
Write-Host "✓ Portable package created: GenXLink-v0.1.0-Portable-Windows-x64.zip" -ForegroundColor Green
Write-Host ""

# Step 4: Check for NSIS
Write-Host "[4/5] Checking for NSIS installer..." -ForegroundColor Yellow
$nsisPath = "C:\Program Files (x86)\NSIS\makensis.exe"
if (Test-Path $nsisPath) {
    Write-Host "✓ NSIS found, building installer..." -ForegroundColor Green
    & $nsisPath "installer/genxlink-installer.nsi"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ NSIS installer created: GenXLink-v0.1.0-Setup-Windows-x64.exe" -ForegroundColor Green
    } else {
        Write-Host "⚠ NSIS build had warnings (check output above)" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠ NSIS not found at: $nsisPath" -ForegroundColor Yellow
    Write-Host "  Download from: https://nsis.sourceforge.io/Download" -ForegroundColor Yellow
    Write-Host "  Installer will not be created (portable package is ready)" -ForegroundColor Yellow
}
Write-Host ""

# Step 5: Summary
Write-Host "[5/5] Build Summary" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$portableZip = "dist/GenXLink-v0.1.0-Portable-Windows-x64.zip"
$installerExe = "dist/GenXLink-v0.1.0-Setup-Windows-x64.exe"

if (Test-Path $portableZip) {
    $size = (Get-Item $portableZip).Length / 1MB
    Write-Host "✓ Portable Package:" -ForegroundColor Green
    Write-Host "  File: $portableZip" -ForegroundColor White
    Write-Host "  Size: $([math]::Round($size, 2)) MB" -ForegroundColor White
    Write-Host ""
}

if (Test-Path $installerExe) {
    $size = (Get-Item $installerExe).Length / 1MB
    Write-Host "✓ Installer Package:" -ForegroundColor Green
    Write-Host "  File: $installerExe" -ForegroundColor White
    Write-Host "  Size: $([math]::Round($size, 2)) MB" -ForegroundColor White
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "✓ Build Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Distribution files are in the 'dist' folder" -ForegroundColor White
Write-Host "Ready for distribution!" -ForegroundColor Green
Write-Host ""
Write-Host "Copyright (c) 2025 GenXis Innovations" -ForegroundColor Gray
Write-Host "Contact: genxisinnovation@outlook.com" -ForegroundColor Gray
