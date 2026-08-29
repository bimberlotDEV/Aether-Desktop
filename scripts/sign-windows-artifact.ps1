[CmdletBinding()]
param(
  [Parameter(Mandatory = $true, Position = 0)]
  [string]$ArtifactPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-RequiredEnvironmentValue([string]$Name) {
  $value = [Environment]::GetEnvironmentVariable($Name)
  if ([string]::IsNullOrWhiteSpace($value)) {
    throw "$Name is required for Windows release signing."
  }
  return $value.Trim()
}

$endpoint = Get-RequiredEnvironmentValue 'AZURE_ARTIFACT_SIGNING_ENDPOINT'
$account = Get-RequiredEnvironmentValue 'AZURE_ARTIFACT_SIGNING_ACCOUNT'
$profile = Get-RequiredEnvironmentValue 'AZURE_ARTIFACT_SIGNING_PROFILE'

$endpointUri = $null
if (-not [Uri]::TryCreate($endpoint, [UriKind]::Absolute, [ref]$endpointUri) -or
    $endpointUri.Scheme -ne 'https' -or
    -not $endpointUri.Host.EndsWith('.codesigning.azure.net', [StringComparison]::OrdinalIgnoreCase)) {
  throw 'AZURE_ARTIFACT_SIGNING_ENDPOINT must be an HTTPS Azure Artifact Signing endpoint.'
}

foreach ($value in @($account, $profile)) {
  if ($value -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') {
    throw 'Artifact Signing account and profile names may contain only letters, numbers, dot, underscore, and hyphen.'
  }
}

$resolvedArtifact = (Resolve-Path -LiteralPath $ArtifactPath).Path
$extension = [System.IO.Path]::GetExtension($resolvedArtifact).ToLowerInvariant()
if ($extension -notin @('.exe', '.msi', '.dll')) {
  throw "Refusing to sign unsupported artifact type '$extension'."
}

$signer = Get-Command 'artifact-signing-cli' -CommandType Application -ErrorAction Stop
& $signer.Source sign '-e' $endpoint '-a' $account '-c' $profile '-d' 'Aether' $resolvedArtifact
if ($LASTEXITCODE -ne 0) {
  throw "Windows signing failed for $([System.IO.Path]::GetFileName($resolvedArtifact))."
}
