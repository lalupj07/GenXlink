# Build-MSIX.ps1
# Script to build MSIX package for Microsoft Store

param(
    [string]$Version = "1.0.0.0",
    [string]$Publisher = "CN=GenXis Innovations"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
if (-not $ProjectRoot -or $ProjectRoot -eq "") { 
    $ProjectRoot = Get-Location 
}

Write-Host "[STORE] Building MSIX Package for Microsoft Store" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

# Paths
$MsixDir = Join-Path $ProjectRoot "msix-package"
$AssetsDir = Join-Path $MsixDir "Assets"
$DistDir = Join-Path $ProjectRoot "dist"
$ExePath = Join-Path $ProjectRoot "dist\windows-portable\GenXLink.exe"
$IcoPath = Join-Path $ProjectRoot "assets\icons\genxlink.ico"

# Create directories
New-Item -ItemType Directory -Force -Path $AssetsDir | Out-Null

# Check if source files exist
if (-not (Test-Path $ExePath)) {
    Write-Host "[ERROR] GenXLink.exe not found at: $ExePath" -ForegroundColor Red
    Write-Host "   Please build the project first: cargo build --release -p genxlink-ui-demo" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Found GenXLink.exe" -ForegroundColor Green

# Copy executable to MSIX package
Copy-Item $ExePath -Destination $MsixDir -Force
Write-Host "[OK] Copied executable to MSIX package" -ForegroundColor Green

# Generate placeholder PNG assets (in production, use proper image conversion)
# For now, create simple colored PNGs as placeholders
$sizes = @{
    "Square44x44Logo" = 44
    "Square71x71Logo" = 71
    "Square150x150Logo" = 150
    "Square310x310Logo" = 310
    "Wide310x150Logo" = @(310, 150)
    "StoreLogo" = 50
    "SplashScreen" = @(620, 300)
}

Write-Host "[INFO] Creating placeholder assets..." -ForegroundColor Yellow
Write-Host "   Note: Replace these with proper branded images before Store submission" -ForegroundColor Yellow

# Create simple placeholder PNGs using .NET
Add-Type -AssemblyName System.Drawing

foreach ($name in $sizes.Keys) {
    $size = $sizes[$name]
    if ($size -is [array]) {
        $width = $size[0]
        $height = $size[1]
    } else {
        $width = $size
        $height = $size
    }
    
    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    
    # Fill with brand color
    $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(26, 26, 46))
    $graphics.FillRectangle($brush, 0, 0, $width, $height)
    
    # Add "GX" text
    $minSize = [Math]::Min($width, $height)
    $fontSize = [Math]::Max(8, $minSize / 3)
    $font = New-Object System.Drawing.Font("Arial", $fontSize, [System.Drawing.FontStyle]::Bold)
    $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0, 212, 255))
    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rect = New-Object System.Drawing.RectangleF(0, 0, $width, $height)
    $graphics.DrawString("GX", $font, $textBrush, $rect, $format)
    
    $outputPath = Join-Path $AssetsDir "$name.png"
    $bitmap.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    
    $graphics.Dispose()
    $bitmap.Dispose()
    
    Write-Host "   Created: $name.png (${width}x${height})" -ForegroundColor Gray
}

Write-Host "[OK] Created placeholder assets" -ForegroundColor Green

# Find MakeAppx.exe
$sdkPath = "C:\Program Files (x86)\Windows Kits\10\bin"
$makeappx = Get-ChildItem -Path $sdkPath -Recurse -Filter "makeappx.exe" -ErrorAction SilentlyContinue | 
    Where-Object { $_.FullName -match "x64" } | 
    Select-Object -First 1

if (-not $makeappx) {
    Write-Host "[ERROR] MakeAppx.exe not found. Please install Windows SDK." -ForegroundColor Red
    Write-Host "   Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Found MakeAppx at: $($makeappx.FullName)" -ForegroundColor Green

# Create MSIX package
$msixOutput = Join-Path $DistDir "GenXLink-v1.0.0.msix"
Write-Host "[INFO] Creating MSIX package..." -ForegroundColor Cyan

& $makeappx.FullName pack /d $MsixDir /p $msixOutput /o

if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] MSIX package created: $msixOutput" -ForegroundColor Green
    $size = (Get-Item $msixOutput).Length / 1MB
    Write-Host "   Size: $([Math]::Round($size, 2)) MB" -ForegroundColor Gray
} else {
    Write-Host "[ERROR] Failed to create MSIX package" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[SUCCESS] MSIX Package Ready!" -ForegroundColor Green
Write-Host ""
Write-Host "Next Steps for Microsoft Store:" -ForegroundColor Cyan
Write-Host "1. Create a Microsoft Partner Center account: https://partner.microsoft.com" -ForegroundColor White
Write-Host "2. Reserve your app name 'GenXLink'" -ForegroundColor White
Write-Host "3. Update Publisher in AppxManifest.xml with your actual Publisher ID" -ForegroundColor White
Write-Host "4. Replace placeholder assets with proper branded images" -ForegroundColor White
Write-Host "5. Sign the package with your certificate" -ForegroundColor White
Write-Host "6. Upload to Partner Center for certification" -ForegroundColor White
