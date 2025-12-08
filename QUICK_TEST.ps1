# GenXLink - Quick Interactive Test
# Run this to verify the application is working

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "GenXLink Quick Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if app is running
$process = Get-Process genxlink -ErrorAction SilentlyContinue

if ($process) {
    Write-Host "✓ Application is running!" -ForegroundColor Green
    Write-Host "  Process ID: $($process.Id)" -ForegroundColor White
    Write-Host "  Memory: $([math]::Round($process.WorkingSet/1MB, 2)) MB" -ForegroundColor White
    Write-Host "  CPU Time: $($process.CPU) seconds" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "✗ Application is not running!" -ForegroundColor Red
    Write-Host "Please start the application first." -ForegroundColor Yellow
    exit 1
}

# Interactive test questions
Write-Host "Please answer the following questions:" -ForegroundColor Cyan
Write-Host ""

# Question 1
Write-Host "1. Can you see the GenXLink window?" -ForegroundColor Yellow
$q1 = Read-Host "   (y/n)"

# Question 2
Write-Host ""
Write-Host "2. Is the dark theme applied correctly?" -ForegroundColor Yellow
$q2 = Read-Host "   (y/n)"

# Question 3
Write-Host ""
Write-Host "3. Can you see 'Settings' and 'Premium' buttons?" -ForegroundColor Yellow
$q3 = Read-Host "   (y/n)"

# Question 4
Write-Host ""
Write-Host "4. Click 'Settings' - does the panel open?" -ForegroundColor Yellow
$q4 = Read-Host "   (y/n)"

# Question 5
Write-Host ""
Write-Host "5. Can you see language dropdown and sliders?" -ForegroundColor Yellow
$q5 = Read-Host "   (y/n)"

# Question 6
Write-Host ""
Write-Host "6. Click 'Premium' - do you see pricing cards?" -ForegroundColor Yellow
$q6 = Read-Host "   (y/n)"

# Question 7
Write-Host ""
Write-Host "7. Is the text sharp and readable at 150% DPI?" -ForegroundColor Yellow
$q7 = Read-Host "   (y/n)"

# Question 8
Write-Host ""
Write-Host "8. Does the window resize smoothly?" -ForegroundColor Yellow
$q8 = Read-Host "   (y/n)"

# Calculate results
$total = 8
$passed = 0
if ($q1 -eq 'y') { $passed++ }
if ($q2 -eq 'y') { $passed++ }
if ($q3 -eq 'y') { $passed++ }
if ($q4 -eq 'y') { $passed++ }
if ($q5 -eq 'y') { $passed++ }
if ($q6 -eq 'y') { $passed++ }
if ($q7 -eq 'y') { $passed++ }
if ($q8 -eq 'y') { $passed++ }

$passRate = [math]::Round(($passed / $total) * 100, 0)

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Test Results" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Total Tests: $total" -ForegroundColor White
Write-Host "Passed: $passed" -ForegroundColor Green
Write-Host "Failed: $($total - $passed)" -ForegroundColor Red
Write-Host "Pass Rate: $passRate%" -ForegroundColor $(if ($passRate -ge 80) { "Green" } else { "Yellow" })
Write-Host ""

if ($passRate -ge 80) {
    Write-Host "✓ READY FOR SUBMISSION!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "1. Install Inno Setup" -ForegroundColor White
    Write-Host "2. Run: .\CREATE_INSTALLER.ps1" -ForegroundColor White
    Write-Host "3. Test installer" -ForegroundColor White
    Write-Host "4. Submit for certification" -ForegroundColor White
} else {
    Write-Host "⚠ NEEDS ATTENTION" -ForegroundColor Yellow
    Write-Host "Please review failed tests before submission." -ForegroundColor Yellow
}

Write-Host ""

# Save results
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$results = @"
GenXLink Quick Test Results
Date: $timestamp
Process ID: $($process.Id)
Memory: $([math]::Round($process.WorkingSet/1MB, 2)) MB
CPU Time: $($process.CPU) seconds

Test Results:
1. Window visible: $q1
2. Dark theme: $q2
3. Buttons visible: $q3
4. Settings panel: $q4
5. Settings controls: $q5
6. Premium panel: $q6
7. DPI scaling: $q7
8. Window resize: $q8

Total: $passed/$total ($passRate%)
Status: $(if ($passRate -ge 80) { "READY" } else { "NEEDS WORK" })
"@

$results | Out-File "test-results.txt" -Encoding UTF8
Write-Host "Results saved to: test-results.txt" -ForegroundColor Green
Write-Host ""
