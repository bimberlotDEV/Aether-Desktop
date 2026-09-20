[CmdletBinding()]
param(
  [switch]$TestMode
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$outputPath = Join-Path $repositoryRoot 'src-tauri\tauri.release.generated.json'
$publicKey = [Environment]::GetEnvironmentVariable('AETHER_UPDATER_PUBLIC_KEY')
$stableEndpoint = 'https://github.com/bimberlotDEV/Aether-Desktop/releases/latest/download/latest.json'

if ([string]::IsNullOrWhiteSpace($publicKey)) {
  throw 'AETHER_UPDATER_PUBLIC_KEY is required.'
}

$normalizedKey = $publicKey.Trim()
try {
  $decodedKey = [System.Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($normalizedKey))
} catch {
  throw 'AETHER_UPDATER_PUBLIC_KEY is not valid base64 Tauri public-key content.'
}
$keyLines = $decodedKey.Replace("`r`n", "`n").Trim() -split "`n"
if ($normalizedKey -match '\s' -or
    $keyLines.Count -ne 2 -or
    $keyLines[0] -notmatch '^untrusted comment: minisign public key:' -or
    $keyLines[1] -notmatch '^R[WU][A-Za-z0-9+/]{40,}={0,2}$') {
  throw 'AETHER_UPDATER_PUBLIC_KEY is not a complete Tauri minisign public key.'
}

$windowsConfig = @{}
if (-not $TestMode) {
  $windowsConfig.signCommand = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File ../scripts/sign-windows-artifact.ps1 %1'
}

$config = [ordered]@{
  bundle = [ordered]@{
    createUpdaterArtifacts = $true
    windows = $windowsConfig
  }
  plugins = [ordered]@{
    updater = [ordered]@{
      pubkey = $normalizedKey
      endpoints = @($stableEndpoint)
      windows = [ordered]@{
        installMode = 'passive'
      }
    }
  }
}

$json = $config | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText($outputPath, $json + [Environment]::NewLine, [System.Text.UTF8Encoding]::new($false))

Write-Output "Generated release configuration at src-tauri/tauri.release.generated.json."
if ($TestMode) {
  Write-Output 'Test mode: Windows Authenticode signing command is intentionally omitted.'
}
