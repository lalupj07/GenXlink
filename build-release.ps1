# GenXLink Release Build Script
# Run this script to build the Windows application for certification

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "GenXLink Release Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Clean previous builds
Write-Host "[1/5] Cleaning previous builds..." -ForegroundColor Yellow
cargo clean
if ($LASTEXITCODE -ne 0) {
    Write-Host "Warning: cargo clean failed, continuing..." -ForegroundColor Yellow
}

# Step 2: Build the Windows client in release mode
Write-Host ""
Write-Host "[2/5] Building Windows client (Release mode)..." -ForegroundColor Yellow
cargo build --release -p genxlink-windows

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "ERROR: Build failed!" -ForegroundColor Red
    Write-Host "Please fix compilation errors and try again." -ForegroundColor Red
    exit 1
}

# Step 3: Check if executable exists
Write-Host ""
Write-Host "[3/5] Verifying build output..." -ForegroundColor Yellow
$exePath = "target\release\genxlink-windows.exe"

if (Test-Path $exePath) {
    $fileSize = (Get-Item $exePath).Length / 1MB
    Write-Host "✓ Build successful!" -ForegroundColor Green
    Write-Host "  Executable: $exePath" -ForegroundColor Green
    Write-Host "  Size: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Green
} else {
    Write-Host "ERROR: Executable not found at $exePath" -ForegroundColor Red
    exit 1
}

# Step 4: Create dist directory
Write-Host ""
Write-Host "[4/5] Preparing distribution..." -ForegroundColor Yellow
if (-not (Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist" | Out-Null
}

# Copy executable to dist
Copy-Item $exePath "dist\genxlink-windows.exe" -Force
Write-Host "✓ Copied executable to dist\" -ForegroundColor Green

# Step 5: Check for Inno Setup (optional)
Write-Host ""
Write-Host "[5/5] Checking for Inno Setup..." -ForegroundColor Yellow
$innoSetupPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"

if (Test-Path $innoSetupPath) {
    Write-Host "✓ Inno Setup found!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To create installer, run:" -ForegroundColor Cyan
    Write-Host "  & '$innoSetupPath' installer\genxlink-setup.iss" -ForegroundColor White
    Write-Host ""
    
    # Ask if user wants to create installer now
    $response = Read-Host "Create installer now? (y/n)"
    if ($response -eq 'y' -or $response -eq 'Y') {
        Write-Host ""
        Write-Host "Creating installer..." -ForegroundColor Yellow
        & $innoSetupPath "installer\genxlink-setup.iss"
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ Installer created successfully!" -ForegroundColor Green
            Write-Host "  Location: dist\GenXLink-Setup-1.0.0.exe" -ForegroundColor Green
        } else {
            Write-Host "ERROR: Installer creation failed!" -ForegroundColor Red
        }
    }
} else {
    Write-Host "! Inno Setup not found" -ForegroundColor Yellow
    Write-Host "  Download from: https://jrsoftware.org/isdl.php" -ForegroundColor Yellow
    Write-Host "  After installing, run the installer script manually" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Test the executable: dist\genxlink-windows.exe" -ForegroundColor White
Write-Host "2. Create installer (if not done): See instructions above" -ForegroundColor White
Write-Host "3. Test on Surface Laptop 5 @ 150% DPI" -ForegroundColor White
Write-Host "4. Submit for certification" -ForegroundColor White
Write-Host ""
