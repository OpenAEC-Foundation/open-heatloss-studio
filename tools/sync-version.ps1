<#
.SYNOPSIS
Sync workspace version from Cargo.toml to tauri.conf.json and frontend/package.json.

.DESCRIPTION
Single source of truth: Cargo.toml [workspace.package] version.
This script reads it and writes to:
  - src-tauri/tauri.conf.json (top-level "version")
  - frontend/package.json (top-level "version")

Compatible with Windows PowerShell 5.1 and PowerShell 7+.
Run from repo root.
#>

$ErrorActionPreference = 'Stop'

if (-not (Test-Path 'Cargo.toml')) {
    throw "Cargo.toml not found in current directory. Run from repo root."
}

# 1. Read version from Cargo.toml workspace
$cargoToml = Get-Content 'Cargo.toml' -Raw
if ($cargoToml -notmatch '(?ms)\[workspace\.package\].*?version\s*=\s*"([^"]+)"') {
    throw "Could not find [workspace.package] version in Cargo.toml"
}
$version = $Matches[1]
Write-Host "Workspace version: $version"

# UTF-8 without BOM (cross-version compatible)
$utf8NoBom = New-Object System.Text.UTF8Encoding $false

# 2. Sync tauri.conf.json
$tauriConfPath = (Resolve-Path 'src-tauri/tauri.conf.json').Path
$tauriConf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
$tauriConf.version = $version
$tauriJson = $tauriConf | ConvertTo-Json -Depth 32
[System.IO.File]::WriteAllText($tauriConfPath, $tauriJson, $utf8NoBom)
Write-Host "Updated src-tauri/tauri.conf.json -> version $version"

# 3. Sync frontend/package.json
$pkgPath = (Resolve-Path 'frontend/package.json').Path
$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$pkg.version = $version
$pkgJson = $pkg | ConvertTo-Json -Depth 32
[System.IO.File]::WriteAllText($pkgPath, $pkgJson, $utf8NoBom)
Write-Host "Updated frontend/package.json -> version $version"

Write-Host "Version sync complete: $version"
