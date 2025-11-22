# GenXLink Clean Script
# Removes all build artifacts and dependencies

Write-Host "=== GenXLink Clean Script ===" -ForegroundColor Cyan
Write-Host ""

$confirmation = Read-Host "This will remove all build artifacts. Continue? (y/n)"

if ($confirmation -ne 'y') {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow

# Clean cargo build
cargo clean

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Cargo clean successful" -ForegroundColor Green
} else {
    Write-Host "✗ Cargo clean failed" -ForegroundColor Red
    exit 1
}

# Calculate space freed
$targetDir = "target"
if (Test-Path $targetDir) {
    $size = (Get-ChildItem $targetDir -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB
    Write-Host "Freed: $([math]::Round($size, 2)) MB" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== Clean Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "To rebuild:" -ForegroundColor Green
Write-Host "  cargo build --workspace"
Write-Host ""
