Write-Host "Starting Pollen DEK Local Control Plane..." -ForegroundColor Cyan

$DistPath = "apps\local-admin-dashboard\dist"
if (-not (Test-Path $DistPath)) {
    Write-Host "Building Local Admin Dashboard for the first time..." -ForegroundColor Yellow
    Push-Location "apps\local-admin-dashboard"
    npm install
    npm run build
    Pop-Location
}

Write-Host "Compiling the Local Control Plane..." -ForegroundColor Yellow
cargo build -p local-control-plane --release

$existing = Get-Process -Name "local-control-plane" -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Stopping existing Local Control Plane process..." -ForegroundColor Yellow
    Stop-Process -Name "local-control-plane" -Force
}

Write-Host "Starting Local Control Plane in background..." -ForegroundColor Yellow
$env:DEK_LCP_AUTH_DISABLE="1"
Start-Process -FilePath "target\release\local-control-plane.exe" -WindowStyle Hidden

Write-Host "Waiting for server to start..."
Start-Sleep -Seconds 2
Write-Host "Opening Dashboard at http://localhost:3000" -ForegroundColor Green
Start-Process "http://localhost:3000"

Write-Host "Done! The Local Control Plane is now running silently in the background." -ForegroundColor Cyan
Write-Host "To stop it, run: .\stop-dek.ps1" -ForegroundColor Gray
