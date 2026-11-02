# ==============================================================================
#  Nityaksha AI Agent Control Plane CLI (ntx) Windows PowerShell Installer
#  Domain: simhaonline.ai
#  Usage:  powershell -ExecutionPolicy Bypass -c "irm https://simhaonline.ai/install.ps1 | iex"
#  Supports: Windows 10/11, Windows Server (x64, ARM64)
# ==============================================================================

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "         NITYAKSHA AI AGENT & BOT CONTROL PLANE (v1.0.0 by Simha)               " -ForegroundColor Cyan
Write-Host "         Windows Pure Native Rust Installer (simhaonline.ai)                    " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$InstallDir = Join-Path $env:USERPROFILE ".nityaksha\bin"
$DataDir    = Join-Path $env:USERPROFILE ".nityaksha\db"
$CasDir     = Join-Path $env:USERPROFILE ".nityaksha\cas-cache"
$WsDir      = Join-Path $env:USERPROFILE ".nityaksha\workspaces"

# Ensure directories exist
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
New-Item -ItemType Directory -Force -Path $CasDir | Out-Null
New-Item -ItemType Directory -Force -Path $WsDir | Out-Null

$ExePath = Join-Path $InstallDir "ntx.exe"

$ReleaseUrl = "https://github.com/simhaonline/ntx/releases/latest/download/ntx.exe"
$RawUrl     = "https://raw.githubusercontent.com/simhaonline/ntx/main/ntx.exe"

Write-Host "[1/4] Fetching native 'ntx.exe' pure Rust binary for Windows..." -ForegroundColor Cyan

$DownloadOk = $false

try {
    Invoke-WebRequest -Uri $RawUrl -OutFile $ExePath -UseBasicParsing -TimeoutSec 15
    $DownloadOk = $true
} catch {
    try {
        Invoke-WebRequest -Uri $ReleaseUrl -OutFile $ExePath -UseBasicParsing -TimeoutSec 15
        $DownloadOk = $true
    } catch {
        Write-Host "[WARN] Remote precompiled binary download failed. Checking for local cargo compiler..." -ForegroundColor Yellow
    }
}

if (-not $DownloadOk) {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "[INFO] Compiling from source via cargo..." -ForegroundColor Cyan
        cargo install --git https://github.com/simhaonline/ntx.git --root "$env:USERPROFILE\.cargo" ntx
        Copy-Item "$env:USERPROFILE\.cargo\bin\ntx.exe" $ExePath -Force
        $DownloadOk = $true
    } else {
        Write-Host "[ERROR] Could not download ntx.exe and cargo compiler is not installed." -ForegroundColor Red
        Write-Host "Please visit https://github.com/simhaonline/ntx to download manually." -ForegroundColor Red
        exit 1
    }
}

Write-Host "[2/4] Initializing Nityaksha local environment..." -ForegroundColor Cyan

# Configure policy file if not present
$PolicyDir = Join-Path $env:USERPROFILE ".nityaksha"
$PolicyFile = Join-Path $PolicyDir "policy.yaml"
if (-not (Test-Path $PolicyFile)) {
    @"
apiVersion: nityaksha.simha/v1
kind: SecurityPolicy
metadata:
  name: default-windows-policy
spec:
  sandbox:
    workspace: "$WsDir"
    network: true
    brokerMutations: gated
"@ | Set-Content -Path $PolicyFile
}

Write-Host "[3/4] Updating User PATH environment variable..." -ForegroundColor Cyan

$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$InstallDir;$UserPath"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$InstallDir;$env:Path"
    Write-Host "[INFO] Added $InstallDir to User PATH." -ForegroundColor Green
} else {
    Write-Host "[INFO] $InstallDir already in User PATH." -ForegroundColor Gray
}

Write-Host "[4/4] Verifying Windows installation..." -ForegroundColor Cyan

try {
    & $ExePath --version
    Write-Host ""
    Write-Host "✓ Nityaksha Control Plane (ntx.exe) installed successfully!" -ForegroundColor Green
    Write-Host "Binary Location: $ExePath" -ForegroundColor White
    Write-Host ""
    Write-Host "Quick Start Commands:" -ForegroundColor White
    Write-Host "  ntx doctor            - Verify system invariants and memory stack"
    Write-Host "  ntx status            - View system status and registered capabilities"
    Write-Host "  ntx skill list        - List registered proprietary skills"
    Write-Host "  ntx harness status    - View AI execution harness & HITL gates"
    Write-Host "  ntx ui --port 8080    - Launch workstation WebUI"
    Write-Host "  ntx repl              - Enter interactive shell"
    Write-Host ""
    Write-Host "Documentation: https://simhaonline.ai | https://github.com/simhaonline/ntx" -ForegroundColor Cyan
} catch {
    Write-Host "[ERROR] Verification failed: $_" -ForegroundColor Red
    exit 1
}
