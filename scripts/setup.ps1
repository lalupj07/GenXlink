# GenXLink Development Environment Setup Script

Write-Host "=== GenXLink Development Setup ===" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check Rust
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "✓ Rust installed" -ForegroundColor Green
    cargo --version
    rustc --version
} else {
    Write-Host "✗ Rust not found!" -ForegroundColor Red
    Write-Host "Install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check Git
if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Host "✓ Git installed" -ForegroundColor Green
} else {
    Write-Host "⚠ Git not found (optional)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Installing Rust components..." -ForegroundColor Yellow

# Install clippy and rustfmt
rustup component add clippy rustfmt

Write-Host ""
Write-Host "Checking workspace..." -ForegroundColor Yellow

# Check if in correct directory
if (Test-Path "Cargo.toml") {
    Write-Host "✓ Found Cargo.toml" -ForegroundColor Green
} else {
    Write-Host "✗ Not in project root!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Building project..." -ForegroundColor Yellow

# Build workspace
cargo build --workspace

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Build successful!" -ForegroundColor Green
} else {
    Write-Host "✗ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Setup Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Green
Write-Host "1. Run tests: cargo test --workspace"
Write-Host "2. Test screen capture: cd client\windows && cargo run --example screen_capture_test"
Write-Host "3. Run client: cd client\windows && cargo run"
Write-Host ""
Write-Host "Documentation:" -ForegroundColor Green
Write-Host "- QUICKSTART.md - Quick start guide"
Write-Host "- TEST_INSTRUCTIONS.md - Testing guide"
Write-Host "- PROGRESS.md - Current status"
Write-Host "- docs/ - Full documentation"
Write-Host ""
Write-Host "Happy coding! 🚀" -ForegroundColor Cyan
