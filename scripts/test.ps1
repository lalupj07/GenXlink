# GenXLink Test Runner Script

param(
    [switch]$All,
    [switch]$Unit,
    [switch]$Integration,
    [switch]$Performance,
    [switch]$ScreenCapture,
    [switch]$Verbose
)

Write-Host "=== GenXLink Test Runner ===" -ForegroundColor Cyan
Write-Host ""

$verboseFlag = if ($Verbose) { "-- --nocapture" } else { "" }

if ($All -or (!$Unit -and !$Integration -and !$Performance -and !$ScreenCapture)) {
    Write-Host "Running all tests..." -ForegroundColor Yellow
    cargo test --workspace $verboseFlag
    
    Write-Host ""
    Write-Host "Running screen capture example..." -ForegroundColor Yellow
    Push-Location client\windows
    cargo run --example screen_capture_test
    Pop-Location
}
else {
    if ($Unit) {
        Write-Host "Running unit tests..." -ForegroundColor Yellow
        cargo test --workspace --lib $verboseFlag
    }
    
    if ($Integration) {
        Write-Host "Running integration tests..." -ForegroundColor Yellow
        cargo test --workspace --test '*' $verboseFlag
    }
    
    if ($Performance) {
        Write-Host "Running performance tests..." -ForegroundColor Yellow
        cargo test --workspace performance $verboseFlag
    }
    
    if ($ScreenCapture) {
        Write-Host "Running screen capture test..." -ForegroundColor Yellow
        Push-Location client\windows
        cargo run --example screen_capture_test
        Pop-Location
    }
}

Write-Host ""
Write-Host "=== Tests Complete ===" -ForegroundColor Cyan
