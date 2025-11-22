# GenXLink Check Script
# Runs format, lint, and test checks

Write-Host "=== GenXLink Check Script ===" -ForegroundColor Cyan
Write-Host ""

$failed = $false

# Format check
Write-Host "Checking code formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Formatting OK" -ForegroundColor Green
} else {
    Write-Host "✗ Formatting issues found" -ForegroundColor Red
    Write-Host "Run: cargo fmt --all" -ForegroundColor Yellow
    $failed = $true
}

Write-Host ""

# Clippy check
Write-Host "Running clippy..." -ForegroundColor Yellow
cargo clippy --workspace -- -D warnings

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Clippy OK" -ForegroundColor Green
} else {
    Write-Host "✗ Clippy warnings found" -ForegroundColor Red
    $failed = $true
}

Write-Host ""

# Test check
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test --workspace

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Tests OK" -ForegroundColor Green
} else {
    Write-Host "✗ Tests failed" -ForegroundColor Red
    $failed = $true
}

Write-Host ""
Write-Host "=== Check Complete ===" -ForegroundColor Cyan

if ($failed) {
    Write-Host "✗ Some checks failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✓ All checks passed!" -ForegroundColor Green
    exit 0
}
