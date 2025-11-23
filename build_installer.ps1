# GenXLink Windows Installer Build Script
# This script builds the Windows client and creates installers

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "   GenXLink Installer Builder" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Check if Rust is installed
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $cargoPath) {
    Write-Host "❌ Cargo not found in PATH!" -ForegroundColor Red
    Write-Host "`nPlease install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    Write-Host "Or add Cargo to your PATH: C:\Users\$env:USERNAME\.cargo\bin" -ForegroundColor Yellow
    exit 1
}

Write-Host "✓ Cargo found: $($cargoPath.Source)" -ForegroundColor Green

# Build the Windows client
Write-Host "`nBuilding Windows client (Release mode)..." -ForegroundColor Yellow
Write-Host "This may take 5-10 minutes on first build..." -ForegroundColor Gray

$buildOutput = cargo build --release --package genxlink-client 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    Write-Host $buildOutput
    exit 1
}

Write-Host "✓ Build successful!" -ForegroundColor Green

# Check if executable exists
$exePath = "target\release\genxlink-client.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "❌ Executable not found at: $exePath" -ForegroundColor Red
    exit 1
}

$exeSize = (Get-Item $exePath).Length / 1MB
Write-Host "✓ Executable size: $([math]::Round($exeSize, 2)) MB" -ForegroundColor Green

# Create output directory
$outputDir = "dist"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

# Copy portable version
Write-Host "`nCreating portable version..." -ForegroundColor Yellow
$portablePath = "$outputDir\GenXLink-Portable.exe"
Copy-Item $exePath $portablePath -Force
Write-Host "✓ Portable version created: $portablePath" -ForegroundColor Green

# Create version info
$version = "0.1.0"
$versionInfo = @"
GenXLink Windows Client
Version: $version
Build Date: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Platform: Windows x64
Type: Portable Executable

Server: https://genxlink-production.up.railway.app
"@

Set-Content -Path "$outputDir\VERSION.txt" -Value $versionInfo

# Create README for portable version
$portableReadme = @"
# GenXLink Portable

## Quick Start

1. Double-click GenXLink-Portable.exe to run
2. The application will connect to the production server
3. Register or login to start using GenXLink

## Features

- Remote Desktop Control
- Screen Sharing
- File Transfer
- Clipboard Sync
- End-to-End Encryption

## System Requirements

- Windows 10/11 (64-bit)
- Internet connection
- 100 MB free disk space

## Support

- Documentation: https://lalupj07.github.io/GenXlink/
- GitHub: https://github.com/lalupj07/GenXlink
- Server Status: https://genxlink-production.up.railway.app/health

## Version

Version: $version
Build Date: $(Get-Date -Format 'yyyy-MM-dd')
"@

Set-Content -Path "$outputDir\README.md" -Value $portableReadme

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "           BUILD COMPLETE!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n📦 Output Files:" -ForegroundColor Yellow
Write-Host "   - $portablePath" -ForegroundColor White
Write-Host "   - $outputDir\VERSION.txt" -ForegroundColor White
Write-Host "   - $outputDir\README.md" -ForegroundColor White

Write-Host "`n✅ Portable version ready!" -ForegroundColor Green
Write-Host "`n📝 Next Steps:" -ForegroundColor Yellow
Write-Host "   1. Test the portable version" -ForegroundColor White
Write-Host "   2. Install WiX Toolset for MSI installer" -ForegroundColor White
Write-Host "   3. Install Inno Setup for EXE installer" -ForegroundColor White

Write-Host "`n🔗 Installer Tools:" -ForegroundColor Yellow
Write-Host "   - WiX Toolset: https://wixtoolset.org/" -ForegroundColor White
Write-Host "   - Inno Setup: https://jrsoftware.org/isinfo.php" -ForegroundColor White

Write-Host "`n========================================`n" -ForegroundColor Cyan
