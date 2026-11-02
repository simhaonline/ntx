# ==============================================================================
#  NITYAKSHA — An AI Concierge Platform Windows Installer (ntx.exe)
#  Engineered for high-velocity, deterministic AI in pure Rust.
#  Usage:  irm https://simhaonline.ai/install.ps1 | iex
# ==============================================================================

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                   NITYAKSHA - An AI Concierge Platform                         " -ForegroundColor Cyan
Write-Host "         Engineered for high-velocity, deterministic AI in pure Rust.           " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$InstallDir = Join-Path $env:USERPROFILE ".nityaksha\bin"
$DataDir    = Join-Path $env:USERPROFILE ".nityaksha\db"
$CasDir     = Join-Path $env:USERPROFILE ".nityaksha\cas-cache"
$WsDir      = Join-Path $env:USERPROFILE ".nityaksha\workspaces"

# Ensure user directories exist
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $DataDir    | Out-Null
New-Item -ItemType Directory -Force -Path $CasDir     | Out-Null
New-Item -ItemType Directory -Force -Path $WsDir      | Out-Null

$ExePath = Join-Path $InstallDir "ntx.exe"

$DownloadSources = @(
    "https://simhaonline.ai/ntx.exe",
    "https://raw.githubusercontent.com/simhaonline/ntx/main/ntx.exe",
    "https://github.com/simhaonline/ntx/releases/latest/download/ntx.exe"
)

Write-Host "[1/4] Downloading native 64-bit Windows binary (ntx.exe)..." -ForegroundColor Cyan

$DownloadOk = $false

foreach ($url in $DownloadSources) {
    try {
        Write-Host "  -> Fetching from $url ..." -ForegroundColor Gray
        $wc = New-Object System.Net.WebClient
        $wc.Headers.Add("User-Agent", "NityakshaInstaller/1.0")
        $wc.DownloadFile($url, $ExePath)
        
        if ((Test-Path $ExePath) -and ((Get-Item $ExePath).Length -gt 500000)) {
            $DownloadOk = $true
            Write-Host "  [OK] Successfully downloaded binary ($( [math]::Round((Get-Item $ExePath).Length / 1MB, 2) ) MB)" -ForegroundColor Green
            break
        }
    } catch {
        # Try next source
    }
}

if (-not $DownloadOk) {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "[INFO] Downloading from git via cargo compiler..." -ForegroundColor Cyan
        cargo install --git https://github.com/simhaonline/ntx.git --root "$env:USERPROFILE\.cargo" ntx
        Copy-Item "$env:USERPROFILE\.cargo\bin\ntx.exe" $ExePath -Force
        $DownloadOk = $true
    } else {
        Write-Host "[ERROR] Could not automatically download ntx.exe." -ForegroundColor Red
        Write-Host "Manual Installation:" -ForegroundColor Yellow
        Write-Host "  1. Download: https://simhaonline.ai/ntx.exe" -ForegroundColor White
        Write-Host "  2. Save to : $ExePath" -ForegroundColor White
        exit 1
    }
}

# Unblock file from Windows SmartScreen Zone.Identifier
try {
    Unblock-File -Path $ExePath -ErrorAction SilentlyContinue
} catch {}

Write-Host "[2/4] Initializing Nityaksha local security policy..." -ForegroundColor Cyan
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
    Write-Host "  [OK] Added $InstallDir to User PATH." -ForegroundColor Green
} else {
    Write-Host "  [OK] $InstallDir already present in PATH." -ForegroundColor Gray
}

Write-Host "[4/4] Verifying Windows installation..." -ForegroundColor Cyan
try {
    & $ExePath --version
    Write-Host ""
    Write-Host "================================================================================" -ForegroundColor Green
    Write-Host "✓ Nityaksha Control Plane (ntx.exe) is ready to use!" -ForegroundColor Green
    Write-Host "================================================================================" -ForegroundColor Green
    Write-Host "Quick Start:" -ForegroundColor Cyan
    Write-Host "  ntx                 - Launch Interactive AI Concierge Shell" -ForegroundColor White
    Write-Host "  ntx auth login google - Authenticate Google Gemini with Google OAuth" -ForegroundColor White
    Write-Host "  ntx provider list   - View 21 preconfigured AI models & backends" -ForegroundColor White
    Write-Host "  ntx status          - Framework health & diagnostic report" -ForegroundColor White
    Write-Host "  ntx ui              - Open local WebUI Workstation on port 8080" -ForegroundColor White
    Write-Host ""
} catch {
    Write-Host "[WARN] Binary installed to $ExePath. Restart terminal to refresh PATH." -ForegroundColor Yellow
}
