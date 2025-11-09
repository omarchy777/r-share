#!/usr/bin/env pwsh
# Quick Start Script PowerShell version

# Check Docker
Write-Host "→ Checking Docker installation..." -ForegroundColor Yellow
try
{
    $dockerVersion = docker --version
    Write-Host "  ✓ Docker found: $dockerVersion" -ForegroundColor Green
}
catch
{
    Write-Host "  ✗ Docker not found! Please install Docker Desktop." -ForegroundColor Red
    exit 1
}

# Check Docker Compose
Write-Host "→ Checking Docker Compose..." -ForegroundColor Yellow
try
{
    $composeVersion = docker-compose --version
    Write-Host "  ✓ Docker Compose found: $composeVersion" -ForegroundColor Green
}
catch
{
    Write-Host "  ✗ Docker Compose not found!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "→ Building R-Share server (this may take 5-10 minutes)..." -ForegroundColor Yellow
docker-compose build

if ($LASTEXITCODE -ne 0)
{
    Write-Host "  ✗ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "  ✓ Build successful!" -ForegroundColor Green
Write-Host ""

Write-Host "→ Starting R-Share server..." -ForegroundColor Yellow
docker-compose up -d

if ($LASTEXITCODE -ne 0)
{
    Write-Host "  ✗ Failed to start server!" -ForegroundColor Red
    exit 1
}

Write-Host "  ✓ Server started!" -ForegroundColor Green
Write-Host ""

# Wait for health check
Write-Host "→ Waiting for server to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

for ($i = 1; $i -le 12; $i++) {
    try
    {
        $response = Invoke-WebRequest -Uri "http://localhost:8080/actuator/health" -UseBasicParsing -TimeoutSec 5 -ErrorAction Stop
        if ($response.StatusCode -eq 200)
        {
            Write-Host "  ✓ Server is healthy!" -ForegroundColor Green
            $healthy = $true
            break
        }
    }
    catch
    {
        Write-Host "   Waiting... ($i/12)" -ForegroundColor Yellow
        Start-Sleep -Seconds 5
    }
}

if (-not $healthy)
{
    Write-Host "  ✗ Server health check failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Check logs with: docker-compose logs rshare-server" -ForegroundColor Yellow
    exit 1
}
