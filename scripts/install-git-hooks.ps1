[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-Git {
  $command = Get-Command git -ErrorAction SilentlyContinue
  if ($command) {
    return $command.Source
  }

  $fallback = 'C:\Program Files\Git\cmd\git.exe'
  if (Test-Path -LiteralPath $fallback) {
    return $fallback
  }

  $codexFallback = Join-Path $env:USERPROFILE '.cache\codex-runtimes\codex-primary-runtime\dependencies\native\git\cmd\git.exe'
  if (Test-Path -LiteralPath $codexFallback) {
    return $codexFallback
  }

  throw 'Git was not found. Install Git for Windows or add git.exe to PATH.'
}

$git = Resolve-Git
$root = (& $git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or -not $root) {
  throw 'Run this script from inside the Aether Git repository.'
}

Set-Location -LiteralPath $root

& $git config --local core.hooksPath .githooks
if ($LASTEXITCODE -ne 0) { throw 'Failed to configure core.hooksPath.' }

& $git config --local push.autoSetupRemote true
if ($LASTEXITCODE -ne 0) { throw 'Failed to configure push.autoSetupRemote.' }

& $git config --local push.default current
if ($LASTEXITCODE -ne 0) { throw 'Failed to configure push.default.' }

Write-Output 'Aether Git automation installed.'
Write-Output 'Every successful commit will now be pushed to its upstream automatically.'
Write-Output 'Use AETHER_SKIP_AUTO_PUSH=1 for a deliberate one-commit local override.'
