[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$Version,
  [switch]$AllowDirty
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$') {
  throw 'Version must be numeric SemVer (for example 0.5.0).'
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$package = Get-Content -Raw (Join-Path $repositoryRoot 'package.json') | ConvertFrom-Json
$tauri = Get-Content -Raw (Join-Path $repositoryRoot 'src-tauri\tauri.conf.json') | ConvertFrom-Json
$cargoToml = Get-Content -Raw (Join-Path $repositoryRoot 'src-tauri\Cargo.toml')
$cargoLock = Get-Content -Raw (Join-Path $repositoryRoot 'src-tauri\Cargo.lock')
$settings = Get-Content -Raw (Join-Path $repositoryRoot 'src\routes\Settings.tsx')

$cargoVersion = [regex]::Match($cargoToml, '(?ms)^\[package\].*?^version\s*=\s*"([^"]+)"').Groups[1].Value
$lockVersion = [regex]::Match($cargoLock, '(?ms)^\[\[package\]\]\s*name\s*=\s*"aether"\s*version\s*=\s*"([^"]+)"').Groups[1].Value
$versions = [ordered]@{
  package = [string]$package.version
  tauri = [string]$tauri.version
  cargo = $cargoVersion
  lock = $lockVersion
}

foreach ($entry in $versions.GetEnumerator()) {
  if ($entry.Value -ne $Version) {
    throw "$($entry.Key) version '$($entry.Value)' does not match requested version '$Version'."
  }
}

if ($settings -notmatch [regex]::Escape("Alpha $Version")) {
  throw "Settings does not identify Alpha $Version."
}

$generatedConfig = Join-Path $repositoryRoot 'src-tauri\tauri.release.generated.json'
$trackedGeneratedConfig = & git -C $repositoryRoot ls-files -- 'src-tauri/tauri.release.generated.json'
if ($LASTEXITCODE -ne 0) {
  throw 'Could not verify whether the generated release configuration is tracked.'
}
if (-not [string]::IsNullOrWhiteSpace(($trackedGeneratedConfig -join "`n"))) {
  throw 'Generated release configuration must never be tracked.'
}

if (-not $AllowDirty) {
  $dirty = & git -C $repositoryRoot status --porcelain --untracked-files=all
  if (-not [string]::IsNullOrWhiteSpace(($dirty -join "`n"))) {
    throw 'Release builds require a clean worktree.'
  }
}

if (Test-Path -LiteralPath $generatedConfig) {
  Write-Output 'Generated release configuration exists and remains ignored/untracked.'
}

Write-Output "Release identity verified for Aether $Version."
