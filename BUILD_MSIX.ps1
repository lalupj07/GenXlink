# GenXLink - Simple MSIX Builder

Write-Host "GenXLink MSIX Package Creator" -ForegroundColor Cyan
Write-Host ""

# Check executable
if (-not (Test-Path "target\release\genxlink.exe")) {
    Write-Host "ERROR: genxlink.exe not found!" -ForegroundColor Red
    Write-Host "Run: cargo build --release --bin genxlink" -ForegroundColor Yellow
    exit 1
}

Write-Host "Creating MSIX package structure..." -ForegroundColor Yellow

# Create directories
$pkgRoot = "msix\package"
$assetsDir = "$pkgRoot\Assets"
New-Item -ItemType Directory -Force -Path $pkgRoot | Out-Null
New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null

# Copy files
Copy-Item "target\release\genxlink.exe" "$pkgRoot\genxlink.exe" -Force
Copy-Item "msix\AppxManifest.xml" "$pkgRoot\AppxManifest.xml" -Force

# Create placeholder assets
$assets = @("Square44x44Logo.png", "Square71x71Logo.png", "Square150x150Logo.png", 
            "Square310x310Logo.png", "Wide310x150Logo.png", "StoreLogo.png", "SplashScreen.png")

foreach ($asset in $assets) {
    New-Item -ItemType File -Path "$assetsDir\$asset" -Force | Out-Null
}

Write-Host "Package structure created!" -ForegroundColor Green
Write-Host ""
Write-Host "IMPORTANT: Replace placeholder assets with actual PNG images" -ForegroundColor Yellow
Write-Host "Location: $assetsDir" -ForegroundColor White
Write-Host ""

# Try to find makeappx
$makeappx = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter "makeappx.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

if (-not $makeappx) {
    Write-Host "Windows SDK not found. Install from:" -ForegroundColor Yellow
    Write-Host "https://developer.microsoft.com/windows/downloads/windows-sdk/" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Package files are ready at: $pkgRoot" -ForegroundColor Green
    exit 0
}

Write-Host "Creating MSIX package..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "dist" | Out-Null

& $makeappx.FullName pack /d $pkgRoot /p "dist\GenXLink.msix" /o

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "SUCCESS! MSIX created: dist\GenXLink.msix" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "1. Add PNG images to: $assetsDir" -ForegroundColor White
    Write-Host "2. Rebuild: .\BUILD_MSIX.ps1" -ForegroundColor White
    Write-Host "3. Test: Add-AppxPackage dist\GenXLink.msix" -ForegroundColor White
    Write-Host "4. Submit to Microsoft Store" -ForegroundColor White
} else {
    Write-Host "ERROR: MSIX creation failed" -ForegroundColor Red
}
