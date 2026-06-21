param (
    [string]$InstallPath = "C:\Program Files\PollenDEK",
    [string]$BootstrapPath = "C:\ProgramData\PollenDEK\bootstrap.json"
)

# 1. Elevate if not admin
if (!([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "Please run as Administrator."
    exit 1
}

Write-Host "Installing Pollen DEK to $InstallPath..."

# 2. Create directories
if (!(Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Force -Path $InstallPath | Out-Null
}
$DataDir = "C:\ProgramData\PollenDEK"
if (!(Test-Path $DataDir)) {
    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
}

# 3. Copy binaries (assumes binaries are in current dir)
Copy-Item ".\dek-core.exe" "$InstallPath\dek-core.exe" -Force
Copy-Item ".\dek-updater.exe" "$InstallPath\dek-updater.exe" -Force
Copy-Item ".\dek-mcp-proxy.exe" "$InstallPath\dek-mcp-proxy.exe" -Force

# 4. Bootstrap Configuration
if (Test-Path $BootstrapPath) {
    Copy-Item $BootstrapPath "$DataDir\bootstrap.json" -Force
} else {
    Write-Host "Warning: bootstrap.json not found at $BootstrapPath"
}

# 5. Create Windows Service
$ServiceName = "PollenDEKCore"
$ServiceDisplay = "Pollen DEK Core Service"
$ServiceDescription = "Pollen Distributed Enforcement Kernel"

# Check if service exists
$service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($service) {
    Stop-Service -Name $ServiceName -Force
    sc.exe delete $ServiceName
}

# Install service (runs dek-core.exe)
# dek-core will use standard paths. It can detect it's running as a service.
New-Service -Name $ServiceName -DisplayName $ServiceDisplay -BinaryPathName "$InstallPath\dek-core.exe" -Description $ServiceDescription -StartupType Automatic

# Start service
Start-Service -Name $ServiceName

Write-Host "Installation Complete. Pollen DEK Core is now running as a Windows Service."
