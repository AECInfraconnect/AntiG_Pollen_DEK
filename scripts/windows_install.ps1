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

# Apply icacls to lock down C:\ProgramData\PollenDEK
Write-Host "Locking down permissions on $DataDir..."
icacls "$DataDir" /inheritance:r /grant "SYSTEM:(OI)(CI)F" /grant "Administrators:(OI)(CI)F" /T /C /Q

# Check for Edge WebView2 runtime (for dek-ext-authz if GUI is triggered)
$WebView2RegPath = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
$WebView2UserRegPath = "HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
if (!(Test-Path $WebView2RegPath) -and !(Test-Path $WebView2UserRegPath)) {
    Write-Host "Warning: Edge WebView2 runtime is not installed. dek-ext-authz GUI popups may not work."
} else {
    Write-Host "Edge WebView2 runtime detected."
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
