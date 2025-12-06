# GenXLink Auto-Update Client v1.0.0

param(
    [string]$CurrentVersion = "1.0.0",
    [string]$UpdateServer = "https://updates.genxlink.com"
)

function Get-LatestVersion {
    try {
        $manifest = Invoke-RestMethod -Uri "$UpdateServer/update-manifest.json" -TimeoutSec 10
        return $manifest
    } catch {
        Write-Host "Failed to check for updates: $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

Write-Host "GenXLink Auto-Update Client" -ForegroundColor Cyan
Write-Host "Current Version: $CurrentVersion" -ForegroundColor White

$manifest = Get-LatestVersion
if (-not $manifest) {
    exit 1
}

$latestVersion = $manifest.version
Write-Host "Latest Version: $latestVersion" -ForegroundColor White

if ($latestVersion -eq $CurrentVersion) {
    Write-Host "You are running the latest version" -ForegroundColor Green
} else {
    Write-Host "Update available!" -ForegroundColor Yellow
}

Write-Host "Update Info:" -ForegroundColor Cyan
Write-Host "  Version: $latestVersion" -ForegroundColor White
Write-Host "  Changelog: $($manifest.changelog[0])" -ForegroundColor White
