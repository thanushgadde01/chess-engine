# tools/sprt.ps1 - Self-play match script for SPRT testing
param (
    [string]$FastchessPath = "$PSScriptRoot\fastchess\fastchess.exe",
    [string]$EngineNew = "$PSScriptRoot\bin\engine-play.exe",
    [string]$EngineBase = "$PSScriptRoot\bin\engine-base.exe",
    [string]$Book = "$PSScriptRoot\books\openings.epd",
    [int]$Concurrency = 4,
    [string]$TC = "10+0.1"
)

if (-not (Test-Path $FastchessPath)) {
    Write-Warning "fastchess.exe not found at $FastchessPath. Download fastchess from GitHub releases to enable automated SPRT matches."
    exit 0
}

& $FastchessPath `
    -engine cmd="$EngineNew" name="engine-new" `
    -engine cmd="$EngineBase" name="engine-base" `
    -each tc=$TC proto=uci `
    -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 `
    -concurrency $Concurrency `
    -openings file=$Book format=epd order=random `
    -rounds 500

