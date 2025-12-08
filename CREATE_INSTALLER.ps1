# GenXLink - Create Windows Installer
# Run this script after installing Inno Setup

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "GenXLink Installer Creation Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Inno Setup is installed
$innoPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"

if (-not (Test-Path $innoPath)) {
    Write-Host "ERROR: Inno Setup not found!" -ForegroundColor Red
    Write-Host "Expected location: $innoPath" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please install Inno Setup from:" -ForegroundColor Yellow
    Write-Host "https://jrsoftware.org/isdl.php" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "After installation, run this script again." -ForegroundColor Yellow
    exit 1
}

Write-Host "✓ Inno Setup found!" -ForegroundColor Green
Write-Host ""

# Check if executable exists
if (-not (Test-Path "target\release\genxlink.exe")) {
    Write-Host "ERROR: genxlink.exe not found!" -ForegroundColor Red
    Write-Host "Please build the project first:" -ForegroundColor Yellow
    Write-Host "  cargo build --release --bin genxlink" -ForegroundColor White
    exit 1
}

Write-Host "✓ Executable found!" -ForegroundColor Green
Write-Host ""

# Create installer
Write-Host "Creating installer..." -ForegroundColor Yellow
Write-Host ""

try {
    & $innoPath "installer\genxlink-setup.iss"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host "SUCCESS! Installer Created!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host ""
        
        # Check if installer was created
        $installerPath = "dist\GenXLink-Setup-1.0.0.exe"
        if (Test-Path $installerPath) {
            $installerSize = (Get-Item $installerPath).Length / 1MB
            Write-Host "Installer Details:" -ForegroundColor Cyan
            Write-Host "  Location: $installerPath" -ForegroundColor White
            Write-Host "  Size: $([math]::Round($installerSize, 2)) MB" -ForegroundColor White
            Write-Host ""
            Write-Host "Next Steps:" -ForegroundColor Cyan
            Write-Host "1. Test the installer: .\$installerPath" -ForegroundColor White
            Write-Host "2. Submit for certification" -ForegroundColor White
            Write-Host ""
        } else {
            Write-Host "WARNING: Installer file not found at expected location" -ForegroundColor Yellow
            Write-Host "Check the dist\ folder for the installer" -ForegroundColor Yellow
        }
    } else {
        Write-Host ""
        Write-Host "ERROR: Installer creation failed!" -ForegroundColor Red
        Write-Host "Exit code: $LASTEXITCODE" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host ""
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
