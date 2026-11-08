# tools/gauntlet.ps1 - Match against reference opponent
param (
    [string]$FastchessPath = "$PSScriptRoot\fastchess\fastchess.exe",
    [string]$Engine = "$PSScriptRoot\bin\engine-play.exe",
    [string]$Opponent = "$PSScriptRoot\stockfish\stockfish.exe",
    [string]$Book = "$PSScriptRoot\books\openings.epd",
    [int]$Elo = 2400,
    [int]$Concurrency = 4,
    [string]$TC = "10+0.1"
)

if (-not (Test-Path $FastchessPath)) {
    Write-Warning "fastchess.exe not found at $FastchessPath. Download fastchess to enable gauntlet matches."
    exit 0
}

& $FastchessPath `
    -engine cmd="$Engine" name="RustChess" `
    -engine cmd="$Opponent" name="Stockfish" option.UCI_LimitStrength=true option.UCI_Elo=$Elo `
    -each tc=$TC proto=uci `
    -concurrency $Concurrency `
    -openings file=$Book format=epd order=random `
    -rounds 100

