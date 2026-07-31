[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [ValidatePattern('^(feat|fix|refactor|docs|test|chore|build|ci|perf)(\([^)]+\))?: .+')]
  [string]$Message,

  [Parameter(Mandatory = $true)]
  [ValidateNotNullOrEmpty()]
  [string[]]$Paths
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-Executable {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string[]]$Fallbacks
  )

  $command = Get-Command $Name -ErrorAction SilentlyContinue
  if ($command) { return $command.Source }

  foreach ($fallback in $Fallbacks) {
    if (Test-Path -LiteralPath $fallback) { return $fallback }
  }

  throw "$Name was not found. Install it or add it to PATH."
}

$codexGit = Join-Path $env:USERPROFILE '.cache\codex-runtimes\codex-primary-runtime\dependencies\native\git\cmd\git.exe'
$git = Resolve-Executable -Name 'git' -Fallbacks @('C:\Program Files\Git\cmd\git.exe', $codexGit)
$gh = Resolve-Executable -Name 'gh' -Fallbacks @('C:\Program Files\GitHub CLI\gh.exe')

$root = (& $git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or -not $root) {
  throw 'Run this script from inside the Aether Git repository.'
}

Set-Location -LiteralPath $root

$branch = (& $git branch --show-current).Trim()
if (-not $branch) { throw 'Publishing from detached HEAD is not allowed.' }
if ($branch -in @('main', 'master')) {
  throw "Create a task branch before publishing; direct agent commits to '$branch' are not allowed."
}

$existingStaged = @(& $git diff --cached --name-only)
if ($existingStaged.Count -gt 0) {
  throw "The index already contains staged changes: $($existingStaged -join ', '). Commit or unstage them first."
}

& $git diff --check -- @Paths
if ($LASTEXITCODE -ne 0) { throw 'Whitespace validation failed.' }

& $git add -- @Paths
if ($LASTEXITCODE -ne 0) { throw 'Failed to stage the requested paths.' }

if ($Paths -contains '.githooks/post-commit') {
  & $git update-index --chmod=+x .githooks/post-commit
  if ($LASTEXITCODE -ne 0) { throw 'Failed to mark the post-commit hook executable.' }
}

$staged = @(& $git diff --cached --name-only)
if ($staged.Count -eq 0) { throw 'No changes were staged.' }

$blockedPatterns = @(
  '(^|/)\.env($|\.)',
  '\.(db|db-journal|db-wal|db-shm)$',
  '(^|/)(id_rsa|id_ed25519)(\.pub)?$',
  '\.(pem|p12|pfx|key)$',
  '(^|/)(credentials|secrets?)(\.|/|$)'
)

foreach ($file in $staged) {
  $normalized = $file -replace '\\', '/'
  foreach ($pattern in $blockedPatterns) {
    if ($normalized -match $pattern) {
      & $git restore --staged -- @Paths
      throw "Refusing to publish potentially sensitive file: $file"
    }
  }
}

& $git diff --cached --check
if ($LASTEXITCODE -ne 0) {
  & $git restore --staged -- @Paths
  throw 'Staged diff validation failed.'
}

& $git commit -m $Message
if ($LASTEXITCODE -ne 0) { throw 'Commit failed.' }

# The post-commit hook already attempts this push. Repeating it is intentional:
# it makes push failure visible to this script and is a harmless no-op on success.
& $git push --set-upstream origin $branch
if ($LASTEXITCODE -ne 0) {
  throw 'Push failed. The commit remains safe locally; fix connectivity/authentication and rerun git push.'
}

$env:PATH = "$(Split-Path -Parent $git);$env:PATH"
$repo = & $gh repo view --json nameWithOwner,defaultBranchRef | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve GitHub repository metadata.' }

$defaultBranch = $repo.defaultBranchRef.name
if ($branch -ne $defaultBranch) {
  $existingPrJson = & $gh pr list --head $branch --state open --json number
  if ($LASTEXITCODE -ne 0) { throw 'Could not inspect existing pull requests.' }
  $existingPr = $existingPrJson | ConvertFrom-Json
  if ($null -eq $existingPr -or @($existingPr).Count -eq 0) {
    & $gh pr create --draft --fill --base $defaultBranch --head $branch
    if ($LASTEXITCODE -ne 0) { throw 'Push succeeded, but draft PR creation failed.' }
  } else {
    Write-Output "An open pull request already exists for '$branch'."
  }
}

Write-Output "Published '$Message' from '$branch' to $($repo.nameWithOwner)."
