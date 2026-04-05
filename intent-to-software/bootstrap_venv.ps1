param(
  [string]$Python = "python"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

if (-not (Test-Path ".\\.venv")) {
  & $Python -m venv .venv
}

$venvPython = Join-Path $root ".venv\\Scripts\\python.exe"
& $venvPython -m pip install -U pip
& $venvPython -m pip install -r requirements.txt

Write-Output "Venv ready: $root\\.venv"

