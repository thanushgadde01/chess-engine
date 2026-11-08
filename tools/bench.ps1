# tools/bench.ps1 - Engine Benchmark Script
param (
    [string]$EnginePath = "$PSScriptRoot\bin\engine-play.exe"
)

if (-not (Test-Path $EnginePath)) {
    $EnginePath = "$PSScriptRoot\..\target\release\engine.exe"
}

if (-not (Test-Path $EnginePath)) {
    Write-Error "Engine executable not found at $EnginePath. Run 'cargo build --release' first."
    exit 1
}

Write-Host "Running bench with $EnginePath ..." -ForegroundColor Cyan
"bench`nquit`n" | & $EnginePath

