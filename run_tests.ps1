# GenXLink Comprehensive Test Runner
# PowerShell script to run all tests

$API_URL = "https://genxlink-production.up.railway.app"
$testEmail = "test$(Get-Date -Format 'yyyyMMddHHmmss')@example.com"
$token = $null

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "   GenXLink Comprehensive Test Suite" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$totalTests = 0
$passedTests = 0
$failedTests = 0

function Test-Endpoint {
    param(
        [string]$Name,
        [scriptblock]$TestBlock
    )
    
    $totalTests++
    Write-Host "`n[$totalTests] Testing: $Name" -ForegroundColor Yellow
    
    try {
        & $TestBlock
        $passedTests++
        Write-Host "   ✓ PASS" -ForegroundColor Green
        return $true
    }
    catch {
        $failedTests++
        Write-Host "   ✗ FAIL: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# ============================================================================
# SERVER INFRASTRUCTURE TESTS
# ============================================================================

Write-Host "`n=== SERVER INFRASTRUCTURE TESTS ===" -ForegroundColor Cyan

Test-Endpoint "Server Health Check" {
    $response = Invoke-RestMethod -Uri "$API_URL/health" -Method Get
    if ($response.status -ne "healthy") {
        throw "Server not healthy"
    }
}

Test-Endpoint "Server Info Page" {
    $response = Invoke-WebRequest -Uri "$API_URL/" -Method Get
    if ($response.Content -notlike "*GenXLink*") {
        throw "Invalid server info page"
    }
}

Test-Endpoint "Response Time Check" {
    $start = Get-Date
    Invoke-RestMethod -Uri "$API_URL/health" -Method Get | Out-Null
    $duration = (Get-Date) - $start
    if ($duration.TotalMilliseconds -gt 5000) {
        throw "Slow response: $($duration.TotalMilliseconds)ms"
    }
    Write-Host "   Response time: $([math]::Round($duration.TotalMilliseconds, 2))ms" -ForegroundColor Gray
}

# ============================================================================
# AUTHENTICATION TESTS
# ============================================================================

Write-Host "`n=== AUTHENTICATION TESTS ===" -ForegroundColor Cyan

Test-Endpoint "User Registration" {
    $body = @{
        email = $testEmail
        password = "TestPass123!"
        full_name = "Test User"
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$API_URL/auth/register" -Method Post -Body $body -ContentType "application/json"
    
    if (-not $response.token) {
        throw "No token returned"
    }
    if (-not $response.user) {
        throw "No user data returned"
    }
    
    $script:token = $response.token
    Write-Host "   Token: $($token.Substring(0, 20))..." -ForegroundColor Gray
}

Test-Endpoint "Duplicate Email Prevention" {
    $body = @{
        email = $testEmail
        password = "TestPass123!"
        full_name = "Test User"
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$API_URL/auth/register" -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
        throw "Duplicate email was allowed"
    }
    catch {
        if ($_.Exception.Message -notlike "*400*" -and $_.Exception.Message -notlike "*already*") {
            throw "Wrong error for duplicate email"
        }
    }
}

Test-Endpoint "Password Validation" {
    $body = @{
        email = "test$(Get-Date -Format 'yyyyMMddHHmmss')@example.com"
        password = "short"
        full_name = "Test User"
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$API_URL/auth/register" -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
        throw "Short password was allowed"
    }
    catch {
        if ($_.Exception.Message -notlike "*400*" -and $_.Exception.Message -notlike "*8 characters*") {
            throw "Wrong error for short password"
        }
    }
}

Test-Endpoint "User Login" {
    $body = @{
        email = $testEmail
        password = "TestPass123!"
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$API_URL/auth/login" -Method Post -Body $body -ContentType "application/json"
    
    if (-not $response.token) {
        throw "No token returned"
    }
    
    $script:token = $response.token
}

Test-Endpoint "Invalid Credentials" {
    $body = @{
        email = $testEmail
        password = "WrongPassword123!"
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$API_URL/auth/login" -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
        throw "Invalid credentials were accepted"
    }
    catch {
        if ($_.Exception.Message -notlike "*401*" -and $_.Exception.Message -notlike "*Invalid*") {
            throw "Wrong error for invalid credentials"
        }
    }
}

Test-Endpoint "JWT Token Format" {
    if (-not $token) {
        throw "No token available"
    }
    
    $parts = $token.Split('.')
    if ($parts.Count -ne 3) {
        throw "Invalid JWT format"
    }
}

Test-Endpoint "JWT Token Payload" {
    if (-not $token) {
        throw "No token available"
    }
    
    $payload = $token.Split('.')[1]
    # Add padding if needed
    while ($payload.Length % 4 -ne 0) {
        $payload += "="
    }
    
    $decodedBytes = [System.Convert]::FromBase64String($payload)
    $decodedJson = [System.Text.Encoding]::UTF8.GetString($decodedBytes)
    $claims = $decodedJson | ConvertFrom-Json
    
    if (-not $claims.sub) {
        throw "No user ID in token"
    }
    if (-not $claims.email) {
        throw "No email in token"
    }
    if (-not $claims.exp) {
        throw "No expiration in token"
    }
    
    Write-Host "   User ID: $($claims.sub)" -ForegroundColor Gray
    Write-Host "   Email: $($claims.email)" -ForegroundColor Gray
}

# ============================================================================
# PROTECTED ROUTES TESTS
# ============================================================================

Write-Host "`n=== PROTECTED ROUTES TESTS ===" -ForegroundColor Cyan

Test-Endpoint "Protected Route Without Auth" {
    try {
        Invoke-RestMethod -Uri "$API_URL/api/devices" -Method Get -ErrorAction Stop
        throw "Protected route accessible without auth"
    }
    catch {
        if ($_.Exception.Message -notlike "*401*") {
            throw "Wrong status code for unauthorized access"
        }
    }
}

Test-Endpoint "Protected Route With Valid Token" {
    if (-not $token) {
        throw "No token available"
    }
    
    $headers = @{
        Authorization = "Bearer $token"
    }
    
    $response = Invoke-RestMethod -Uri "$API_URL/api/devices" -Method Get -Headers $headers
    
    if ($response -isnot [Array]) {
        throw "Response is not an array"
    }
}

Test-Endpoint "Protected Route With Invalid Token" {
    $headers = @{
        Authorization = "Bearer invalid_token_here"
    }
    
    try {
        Invoke-RestMethod -Uri "$API_URL/api/devices" -Method Get -Headers $headers -ErrorAction Stop
        throw "Invalid token was accepted"
    }
    catch {
        if ($_.Exception.Message -notlike "*401*") {
            throw "Wrong status code for invalid token"
        }
    }
}

Test-Endpoint "Get Current User (/api/me)" {
    if (-not $token) {
        throw "No token available"
    }
    
    $headers = @{
        Authorization = "Bearer $token"
    }
    
    $response = Invoke-RestMethod -Uri "$API_URL/api/me" -Method Get -Headers $headers
    
    if (-not $response.user_id) {
        throw "No user_id in response"
    }
    
    Write-Host "   User ID: $($response.user_id)" -ForegroundColor Gray
}

Test-Endpoint "Get Devices (/api/devices)" {
    if (-not $token) {
        throw "No token available"
    }
    
    $headers = @{
        Authorization = "Bearer $token"
    }
    
    $response = Invoke-RestMethod -Uri "$API_URL/api/devices" -Method Get -Headers $headers
    
    if ($response -isnot [Array]) {
        throw "Response is not an array"
    }
    
    Write-Host "   Devices found: $($response.Count)" -ForegroundColor Gray
}

# ============================================================================
# DATABASE TESTS
# ============================================================================

Write-Host "`n=== DATABASE TESTS ===" -ForegroundColor Cyan

Test-Endpoint "Database Connection" {
    if (-not $token) {
        throw "No token available"
    }
    
    $headers = @{
        Authorization = "Bearer $token"
    }
    
    Invoke-RestMethod -Uri "$API_URL/api/devices" -Method Get -Headers $headers | Out-Null
}

Test-Endpoint "User Persistence" {
    $body = @{
        email = $testEmail
        password = "TestPass123!"
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$API_URL/auth/login" -Method Post -Body $body -ContentType "application/json"
    
    if (-not $response.token) {
        throw "User not persisted in database"
    }
}

# ============================================================================
# SUMMARY
# ============================================================================

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "           TEST SUMMARY" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`nTotal Tests:  $totalTests" -ForegroundColor White
Write-Host "Passed:       $passedTests" -ForegroundColor Green
Write-Host "Failed:       $failedTests" -ForegroundColor Red

$successRate = [math]::Round(($passedTests / $totalTests) * 100, 2)
Write-Host "Success Rate: $successRate%" -ForegroundColor $(if ($successRate -ge 80) { "Green" } elseif ($successRate -ge 60) { "Yellow" } else { "Red" })

if ($failedTests -eq 0) {
    Write-Host "`n🎉 ALL TESTS PASSED! 🎉" -ForegroundColor Green
} elseif ($failedTests -le 2) {
    Write-Host "`n⚠️  MOSTLY PASSING (Some database tests may fail if table not created)" -ForegroundColor Yellow
} else {
    Write-Host "`n❌ SOME TESTS FAILED" -ForegroundColor Red
}

Write-Host "`n========================================`n" -ForegroundColor Cyan

# Exit with appropriate code
exit $failedTests
