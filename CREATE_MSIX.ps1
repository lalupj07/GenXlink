# GenXLink - Create MSIX Package for Microsoft Store Submission
# This script creates an MSIX package from the built executable

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "GenXLink MSIX Package Creator" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
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

# Create MSIX directory structure
Write-Host "Creating MSIX directory structure..." -ForegroundColor Yellow

$msixRoot = "msix\package"
$assetsDir = "$msixRoot\Assets"

# Create directories
New-Item -ItemType Directory -Force -Path $msixRoot | Out-Null
New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null

Write-Host "✓ Directories created" -ForegroundColor Green
Write-Host ""

# Copy executable
Write-Host "Copying executable..." -ForegroundColor Yellow
Copy-Item "target\release\genxlink.exe" "$msixRoot\genxlink.exe" -Force
Write-Host "✓ Executable copied" -ForegroundColor Green
Write-Host ""

# Copy manifest
Write-Host "Copying manifest..." -ForegroundColor Yellow
Copy-Item "msix\AppxManifest.xml" "$msixRoot\AppxManifest.xml" -Force
Write-Host "✓ Manifest copied" -ForegroundColor Green
Write-Host ""

# Create placeholder assets (you'll need to replace these with actual images)
Write-Host "Creating placeholder assets..." -ForegroundColor Yellow
Write-Host "NOTE: You need to replace these with actual PNG images!" -ForegroundColor Yellow
Write-Host ""

$assetSizes = @{
    "Square44x44Logo.png" = "44x44"
    "Square71x71Logo.png" = "71x71"
    "Square150x150Logo.png" = "150x150"
    "Square310x310Logo.png" = "310x310"
    "Wide310x150Logo.png" = "310x150"
    "StoreLogo.png" = "50x50"
    "SplashScreen.png" = "620x300"
}

foreach ($asset in $assetSizes.Keys) {
    $assetPath = "$assetsDir\$asset"
    # Create empty file as placeholder
    New-Item -ItemType File -Path $assetPath -Force | Out-Null
    Write-Host "  Created placeholder: $asset ($($assetSizes[$asset]))" -ForegroundColor Gray
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "IMPORTANT: Asset Requirements" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "You MUST replace the placeholder assets with actual PNG images:" -ForegroundColor Yellow
Write-Host ""
Write-Host "Required sizes:" -ForegroundColor White
foreach ($asset in $assetSizes.Keys) {
    Write-Host "  $asset - $($assetSizes[$asset]) pixels" -ForegroundColor Gray
}
Write-Host ""

# Check if MakeAppx is available
$makeAppxPath = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\makeappx.exe"
if (-not (Test-Path $makeAppxPath)) {
    # Try to find it
    $makeAppxPath = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter "makeappx.exe" -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName
}

if (-not $makeAppxPath) {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Windows SDK Not Found" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To create the MSIX package, you need Windows SDK installed." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Download from:" -ForegroundColor White
    Write-Host "https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "After installing Windows SDK, run this script again." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Package structure is ready at: $msixRoot" -ForegroundColor Green
    exit 0
}

Write-Host "✓ Windows SDK found!" -ForegroundColor Green
Write-Host ""

# Create MSIX package
Write-Host "Creating MSIX package..." -ForegroundColor Yellow
Write-Host ""

$outputPath = "dist\GenXLink.msix"
New-Item -ItemType Directory -Force -Path "dist" | Out-Null

try {
    & $makeAppxPath pack /d $msixRoot /p $outputPath /o
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host "SUCCESS! MSIX Package Created!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host ""
        
        if (Test-Path $outputPath) {
            $msixSize = (Get-Item $outputPath).Length / 1MB
            Write-Host "Package Details:" -ForegroundColor Cyan
            Write-Host "  Location: $outputPath" -ForegroundColor White
            Write-Host "  Size: $([math]::Round($msixSize, 2)) MB" -ForegroundColor White
            Write-Host ""
            Write-Host "========================================" -ForegroundColor Cyan
            Write-Host "Next Steps for Microsoft Store:" -ForegroundColor Cyan
            Write-Host "========================================" -ForegroundColor Cyan
            Write-Host ""
            Write-Host "1. Replace placeholder assets with actual images" -ForegroundColor White
            Write-Host "2. Sign the MSIX package (required for Store)" -ForegroundColor White
            Write-Host "3. Test installation: Add-AppxPackage $outputPath" -ForegroundColor White
            Write-Host "4. Submit to Microsoft Partner Center" -ForegroundColor White
            Write-Host ""
            Write-Host "Signing Command:" -ForegroundColor Cyan
            Write-Host "  SignTool sign /fd SHA256 /a /f YourCertificate.pfx /p Password $outputPath" -ForegroundColor Gray
            Write-Host ""
        }
    } else {
        Write-Host ""
        Write-Host "ERROR: MSIX creation failed!" -ForegroundColor Red
        Write-Host "Exit code: $LASTEXITCODE" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host ""
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
