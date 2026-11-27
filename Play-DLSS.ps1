# Starts the title on the Series S profile with this fork's settings and, when OptiScaler and
# NVIDIA's DLSS DLL are given, with DLSS in place of the title's FSR upscaler.
#
#   .\Play-DLSS.ps1 -Game D:\MyDump\game_scarlett_master.exe `
#       -OptiScaler D:\Tools\Optiscaler_0.9.5 -Dlss D:\Tools\nvngx_dlss.dll [-DataDir D:\Saves]
#
# Without -OptiScaler/-Dlss: the same settings with the title's own upscaler.
# Build first: cargo build --release --locked -p victory
# Nothing is written to the game's folder: OptiScaler is set up in target\release\opti.
# The OptiScaler setup is adapted from dudegio/Neptune-GDeflate's play.ps1 (GPL-3.0-or-later).
param(
  [Parameter(Mandatory = $true)][string]$Game,  # your own copy's game_scarlett_master.exe
  [string]$OptiScaler = '',                     # folder of an extracted OptiScaler release
  [string]$Dlss = '',                           # NVIDIA's nvngx_dlss.dll (DLSS SDK)
  [string]$DataDir = '',                        # optional save folder (default: %LOCALAPPDATA%\victory\data)
  [switch]$RayTracing,                          # the title's ray tracing: fixes magenta reflections, ~30% slower
  [switch]$WarmShaders,                          # precompile previously used pipelines before play
  [ValidateRange(1,32)][int]$WarmJobs = 2,
  [switch]$Log                                  # write a full log to target\release\last-run.log
)
$ErrorActionPreference = 'Stop'

$runtime = Join-Path $PSScriptRoot 'target\release\victory.exe'
if (-not (Test-Path -LiteralPath $runtime -PathType Leaf)) { throw "No $runtime. Run: cargo build --release --locked -p victory" }
if (-not (Test-Path -LiteralPath $Game -PathType Leaf)) { throw "No $Game." }
if (($OptiScaler -and -not $Dlss) -or ($Dlss -and -not $OptiScaler)) { throw 'DLSS needs both -OptiScaler and -Dlss.' }

function Set-IniValue([string]$Text, [string]$Section, [string]$Key, [string]$Value) {
  $lines = $Text -split "`r?`n"
  $in = $false; $done = $false
  for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match '^\[(.+)\]\s*$') { $in = ($Matches[1] -eq $Section); continue }
    if ($in -and -not $done -and $lines[$i] -match ('^\s*' + [regex]::Escape($Key) + '\s*=')) { $lines[$i] = "$Key=$Value"; $done = $true }
  }
  if (-not $done) { Write-Warning "OptiScaler.ini: no $Key in [$Section]" }
  $lines -join "`r`n"
}

# A copy of victory.exe in a folder of its own with OptiScaler as its dxgi.dll and the DLSS DLL
# beside it. OptiScaler takes the PC FidelityFX calls the title's upscaler is redirected to
# (VICTORY_FSR_DLL) and runs DLSS instead.
function Initialize-Opti([string]$Exe, [string]$OptiDir, [string]$DlssDll) {
  $to = Join-Path (Split-Path $Exe -Parent) 'opti'
  New-Item -ItemType Directory -Force $to | Out-Null
  Get-ChildItem $OptiDir -File | Where-Object { $_.Extension -in '.dll', '.ini' -and $_.Name -ne 'OptiScaler.ini' } | Copy-Item -Destination $to -Force
  Get-ChildItem $OptiDir -Directory | Where-Object { $_.Name -ne 'Licenses' } | Copy-Item -Destination $to -Recurse -Force
  Copy-Item (Join-Path $OptiDir 'OptiScaler.dll') (Join-Path $to 'dxgi.dll') -Force
  Remove-Item (Join-Path $to 'OptiScaler.dll') -Force -ErrorAction SilentlyContinue
  Copy-Item $DlssDll (Join-Path $to 'nvngx_dlss.dll') -Force
  $iniPath = Join-Path $to 'OptiScaler.ini'
  if (-not (Test-Path -LiteralPath $iniPath -PathType Leaf)) {
    Copy-Item -LiteralPath (Join-Path $OptiDir 'OptiScaler.ini') -Destination $iniPath
  }
  $ini = Get-Content -LiteralPath $iniPath -Raw
  $ini = Set-IniValue $ini 'Upscalers' 'Dx12Upscaler' 'dlss'
  $ini = Set-IniValue $ini 'FrameGen' 'Enabled' 'false'
  $ini = Set-IniValue $ini 'FrameGen' 'FGInput' 'nofg'
  $ini = Set-IniValue $ini 'FrameGen' 'FGOutput' 'nofg'
  Set-Content -LiteralPath $iniPath -Value $ini -NoNewline
  Copy-Item $Exe (Join-Path $to 'victory.exe') -Force
  $to
}

# cool_fox's launcher settings (Play-VIctory.cmd), plus this fork's.
$settings = [ordered]@{
  VICTORY_AMBIENT_AIR         = '1.5'
  VICTORY_TEXTURE_NO_WAIT     = '1'
  VICTORY_RELAY               = '2'
  VICTORY_WIDE_BACK           = '1'
  VICTORY_CARD_REHASH_RUNS    = '1'
  VICTORY_JOIN_COPIES         = '1'
  VICTORY_INDIRECT_WINDOW     = '1'
  VICTORY_RELAY_COPY          = '1'
  VICTORY_CARD_STEPS          = '2'
  VICTORY_INDIRECT_BATCH      = '2'
  VICTORY_MAP_THREADS         = '1'
  VICTORY_CARD_AWAKE          = '1'
  VICTORY_SHOW_AFTER          = '1'
  VICTORY_AMBIENT             = '1.0'
  VICTORY_AMBIENT_SHADE       = '0.65,0.6'
  VICTORY_JOIN_PARTS          = '8'
  VICTORY_UPLOAD_AHEAD        = '1'
  VICTORY_LEVELS_ON_DEMAND    = '1'
  VICTORY_CARD_BUFFERS_MIN_KB = '64'
  # This fork: NaN and infinite colours are 0 (black sun, banded sky on NVIDIA).
  VICTORY_NAN_SCRUB           = '1'
  # Recorded draw calls translated on the title's own threads: needed for DLSS, and no slower
  # on Series S (RTX 5060 Ti).
  VICTORY_THIN                = '0'
  VICTORY_CACHE_DIR           = (Join-Path $env:LOCALAPPDATA 'VIctory\cache-pareto')
}
if ($RayTracing) { $settings.VICTORY_BVH_BUILDER = '1'; $settings.VICTORY_BVH_QUARANTINE = '0' }
if ($DataDir) { $settings.VICTORY_DATA_DIR = (Resolve-Path -LiteralPath $DataDir).Path }

$exe = $runtime
if ($OptiScaler) {
  $optiDir = (Resolve-Path -LiteralPath $OptiScaler).Path
  if (-not (Test-Path (Join-Path $optiDir 'OptiScaler.dll'))) { throw "No OptiScaler.dll in $optiDir" }
  $opti = Initialize-Opti $runtime $optiDir (Resolve-Path -LiteralPath $Dlss).Path
  $exe = Join-Path $opti 'victory.exe'
  $settings.VICTORY_FSR_DLL = Join-Path $opti 'amd_fidelityfx_dx12.dll'
}

# Respect a custom cache directory and explicitly clear inherited DLSS state.
if (-not (Test-Path Env:VICTORY_SOFT_SHADOWS)) { $env:VICTORY_SOFT_SHADOWS = '1' }
if ($env:VICTORY_CACHE_DIR) { $settings.VICTORY_CACHE_DIR = $env:VICTORY_CACHE_DIR }
if (-not $OptiScaler) { Remove-Item Env:VICTORY_FSR_DLL -ErrorAction SilentlyContinue }
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
if ($WarmShaders) {
  & $runtime warm --used --jobs $WarmJobs
  if ($LASTEXITCODE -ne 0) { Write-Warning 'Pipeline warm-up failed; continuing with normal compilation.' }
}
if ($Log) { $env:VICTORY_LOG = 'info' } elseif (-not $env:VICTORY_LOG) { $env:VICTORY_LOG = 'warn' }

$Game = (Resolve-Path -LiteralPath $Game).Path
# The runtime logs to stderr; Windows PowerShell would stop on it under 'Stop'.
$ErrorActionPreference = 'Continue'
if ($Log) {
  & $exe run $Game --console series-s --execute *> (Join-Path $PSScriptRoot 'target\release\last-run.log')
} else {
  & $exe run $Game --console series-s --execute
}
exit $LASTEXITCODE
