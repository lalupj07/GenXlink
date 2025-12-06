# GenXLink Docker Deployment (Windows) v1.0.0

Write-Host 'Deploying GenXLink with Docker...' -ForegroundColor Green
Write-Host '==================================' -ForegroundColor Green

Write-Host 'Building Docker images...' -ForegroundColor Yellow
docker-compose build

Write-Host 'Starting services...' -ForegroundColor Yellow
docker-compose up -d

Write-Host 'Waiting for services...' -ForegroundColor Yellow
Start-Sleep -Seconds 30

Write-Host ''
Write-Host '==================================' -ForegroundColor Cyan
Write-Host '     GenXLink Deployment Ready' -ForegroundColor Cyan
Write-Host '==================================' -ForegroundColor Cyan
Write-Host ''
Write-Host 'API Server:      http://localhost:8000' -ForegroundColor Green
Write-Host 'Signaling Server: http://localhost:8081' -ForegroundColor Green
Write-Host ''
Write-Host 'GenXLink deployment complete!' -ForegroundColor Green
