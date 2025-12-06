# GenXLink Server Deployment Script
param(
    [Parameter(Position=0)]
    [string]$Target = "help"
)

Write-Host "GenXLink Deployment Script" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Yellow

if ($Target -eq "railway") {
    Write-Host ""
    Write-Host "Deploying to Railway.app..." -ForegroundColor Cyan
    
    $railwayInstalled = Get-Command railway -ErrorAction SilentlyContinue
    
    if (-not $railwayInstalled) {
        Write-Host ""
        Write-Host "Railway CLI not found. Opening Railway dashboard..." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "To deploy to Railway:" -ForegroundColor Green
        Write-Host "  1. Go to https://railway.app/new"
        Write-Host "  2. Click 'Deploy from GitHub repo'"
        Write-Host "  3. Select your GenXLink repository"
        Write-Host "  4. Railway will auto-detect and deploy"
        Write-Host ""
        Start-Process "https://railway.app/new"
    }
    else {
        Set-Location $PSScriptRoot\..
        railway up
    }
}
elseif ($Target -eq "docker") {
    Write-Host ""
    Write-Host "Deploying with Docker..." -ForegroundColor Cyan
    Set-Location $PSScriptRoot\..
    docker-compose up -d
}
elseif ($Target -eq "render") {
    Write-Host ""
    Write-Host "Deploying to Render.com..." -ForegroundColor Cyan
    Start-Process "https://render.com/deploy"
}
else {
    Write-Host ""
    Write-Host "Usage: .\deploy.ps1 [railway|docker|render]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  railway  - Deploy to Railway.app"
    Write-Host "  docker   - Deploy with Docker Compose"
    Write-Host "  render   - Deploy to Render.com"
}
