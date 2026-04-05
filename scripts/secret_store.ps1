param(
  [Parameter(Mandatory = $true, Position = 0)]
  [ValidateSet('set', 'remove', 'list')]
  [string]$Command,

  [Parameter(Position = 1)]
  [string]$Key,

  [string]$StoreFile = (Join-Path $PSScriptRoot '..\\secrets\\secrets.dpapi.json')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Load-Store([string]$path) {
  if (Test-Path -LiteralPath $path) {
    return (Get-Content -LiteralPath $path -Raw | ConvertFrom-Json)
  }
  return [pscustomobject]@{
    version    = 1
    created_at = (Get-Date).ToString('o')
    secrets    = [pscustomobject]@{}
  }
}

function Save-Store([string]$path, $store) {
  $dir = Split-Path -Parent $path
  if (-not (Test-Path -LiteralPath $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
  ($store | ConvertTo-Json -Depth 10) | Set-Content -LiteralPath $path -Encoding utf8NoBOM
}

$store = Load-Store $StoreFile
if (-not $store.secrets) { $store | Add-Member -NotePropertyName secrets -NotePropertyValue ([pscustomobject]@{}) }

switch ($Command) {
  'list' {
    $props = @($store.secrets.PSObject.Properties.Name) | Sort-Object
    if ($props.Count -eq 0) { Write-Output '(empty)'; exit 0 }
    $props | ForEach-Object { Write-Output $_ }
  }
  'set' {
    if (-not $Key) { throw "Missing Key. Usage: scripts\\secret_store.ps1 set KEY_NAME" }
    $secure = Read-Host "Enter secret for $Key" -AsSecureString
    $cipher = $secure | ConvertFrom-SecureString
    $store.secrets | Add-Member -Force -NotePropertyName $Key -NotePropertyValue $cipher
    Save-Store $StoreFile $store
    Write-Output "Stored: $Key (DPAPI-encrypted) -> $StoreFile"
  }
  'remove' {
    if (-not $Key) { throw "Missing Key. Usage: scripts\\secret_store.ps1 remove KEY_NAME" }
    if ($store.secrets.PSObject.Properties.Name -contains $Key) {
      $null = $store.secrets.PSObject.Properties.Remove($Key)
      Save-Store $StoreFile $store
      Write-Output "Removed: $Key"
    } else {
      Write-Output "Not found: $Key"
    }
  }
}

