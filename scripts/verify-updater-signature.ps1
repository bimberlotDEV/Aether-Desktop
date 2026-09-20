[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$ArtifactPath,
  [Parameter(Mandatory = $true)]
  [string]$SignaturePath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$publicKey = [Environment]::GetEnvironmentVariable('AETHER_UPDATER_PUBLIC_KEY')
if ([string]::IsNullOrWhiteSpace($publicKey)) {
  throw 'AETHER_UPDATER_PUBLIC_KEY is required for signature verification.'
}

$artifact = (Resolve-Path -LiteralPath $ArtifactPath).Path
$signature = (Resolve-Path -LiteralPath $SignaturePath).Path
$verifier = Get-Command 'rsign' -CommandType Application -ErrorAction Stop
$publicTemp = [System.IO.Path]::GetTempFileName()
$signatureTemp = [System.IO.Path]::GetTempFileName()
$tamperedTemp = [System.IO.Path]::GetTempFileName()

try {
  [System.IO.File]::WriteAllBytes(
    $publicTemp,
    [Convert]::FromBase64String($publicKey.Trim())
  )
  [System.IO.File]::WriteAllBytes(
    $signatureTemp,
    [Convert]::FromBase64String((Get-Content -Raw $signature).Trim())
  )

  & $verifier.Source verify '-q' '-p' $publicTemp '-x' $signatureTemp $artifact
  if ($LASTEXITCODE -ne 0) {
    throw "Updater signature verification failed for $([System.IO.Path]::GetFileName($artifact))."
  }

  $bytes = [System.IO.File]::ReadAllBytes($artifact)
  if ($bytes.Length -eq 0) {
    throw 'Updater artifact is empty.'
  }
  $middle = [Math]::Floor($bytes.Length / 2)
  $bytes[$middle] = $bytes[$middle] -bxor 1
  [System.IO.File]::WriteAllBytes($tamperedTemp, $bytes)

  & $verifier.Source verify '-q' '-p' $publicTemp '-x' $signatureTemp $tamperedTemp 2>$null
  if ($LASTEXITCODE -eq 0) {
    throw 'Tampered updater artifact was incorrectly accepted.'
  }

  Write-Output "Verified updater signature and tamper rejection for $([System.IO.Path]::GetFileName($artifact))."
} finally {
  foreach ($temporaryFile in @($publicTemp, $signatureTemp, $tamperedTemp)) {
    Remove-Item -LiteralPath $temporaryFile -Force -ErrorAction SilentlyContinue
  }
}
