# Windows Installer PR 1 — Lokaal werkende installer

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vandaag een werkende Windows `.exe` installer kunnen bouwen op een lokale Windows-machine, met NL wizard en per-user install.

**Architecture:** Tauri v2 NSIS-bundler genereert het `.exe`. Eén bron-of-truth voor versie in `Cargo.toml` workspace; een PowerShell-script sync't die naar `tauri.conf.json` en `frontend/package.json` vóór elke build. Een tweede PowerShell-script orchestreert de volledige build (sync → frontend → tauri bundle → output kopiëren naar `dist/installer/`). Placeholder-icons (een blauw vierkant met "I51") gegenereerd via .NET System.Drawing — vervangbaar zodra echte branding er is.

**Tech Stack:** Tauri v2, NSIS, PowerShell 7 (`pwsh`), Node 22, Rust stable, .NET System.Drawing (Windows built-in).

**Spec reference:** [docs/superpowers/specs/2026-05-08-windows-installer-design.md](../specs/2026-05-08-windows-installer-design.md)

**Niet in deze PR:** GitHub Actions release workflow, frontend update-check, update-banner UI, release-procedure docs. Dat is PR 2.

---

## File Structure

| Bestand | Status | Verantwoordelijkheid |
|---|---|---|
| `.gitignore` | wijzigen | Negeer build-output `dist/installer/` |
| `src-tauri/tauri.conf.json` | wijzigen | NSIS-config (per-user, NL, shortcut-naam, installer-icon) |
| `tools/sync-version.ps1` | nieuw | Lees versie uit `Cargo.toml` workspace; schrijf naar `tauri.conf.json` + `frontend/package.json` |
| `tools/make-placeholder-icon.ps1` | nieuw | Genereer 1024×1024 placeholder PNG (eenmalig) |
| `src-tauri/icons/source.png` | nieuw (gegenereerd) | Bron voor Tauri icon-converter |
| `src-tauri/icons/icon.ico` | nieuw (gegenereerd) | Windows installer + executable icon |
| `src-tauri/icons/32x32.png` | nieuw (gegenereerd) | Tauri-required size |
| `src-tauri/icons/128x128.png` | nieuw (gegenereerd) | Tauri-required size |
| `src-tauri/icons/128x128@2x.png` | nieuw (gegenereerd) | Tauri-required size (256×256) |
| `src-tauri/icons/icon.icns` | nieuw (gegenereerd) | macOS icon (Tauri verwacht dit bestand zelfs op Windows) |
| `tools/build-installer.ps1` | nieuw | Orchestreer: sync versie → build frontend → tauri bundle → kopieer naar `dist/installer/` |
| `docs/building-installer.md` | nieuw | Vereisten, hoe te draaien, troubleshooting |

---

## Pre-flight

- [ ] **Step 0: Verifieer omgeving**

Check vereisten:
```powershell
node --version          # moet ≥ 22
npm --version
cargo --version         # moet ≥ 1.78
rustc --version
pwsh --version          # moet ≥ 7 (PowerShell 7+)
```

Als één faalt: stop. Installeer ontbrekende tooling vóór verder gaan:
- Node 22+: https://nodejs.org/
- Rust: https://rustup.rs/
- PowerShell 7: `winget install Microsoft.PowerShell`

Check ook of MSVC build tools beschikbaar zijn (vereist voor Tauri op Windows):
```powershell
where.exe link.exe
```
Als geen output: installeer "Visual Studio Build Tools" met "Desktop development with C++" workload.

---

## Task 1: `.gitignore` update

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1.1: Voeg `dist/installer/` toe**

Open `.gitignore` en voeg toe aan het eind:
```
# Built installers
dist/installer/
```

- [ ] **Step 1.2: Verifieer**

Run:
```powershell
mkdir -Force dist/installer | Out-Null
New-Item dist/installer/test.exe -ItemType File | Out-Null
git status --short dist/installer/
```
Expected: geen output (de map wordt genegeerd).

Cleanup:
```powershell
Remove-Item -Recurse -Force dist
```

- [ ] **Step 1.3: Commit**

```powershell
git add .gitignore
git commit -m "chore(installer): ignore dist/installer/ build output"
```

---

## Task 2: NSIS-configuratie in `tauri.conf.json`

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 2.1: Voeg NSIS-blok toe**

Huidige `bundle`-sectie:
```json
"bundle": {
  "active": true,
  "targets": "all",
  "externalBin": ["binaries/ifc-tool"],
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ]
}
```

Wijzig naar:
```json
"bundle": {
  "active": true,
  "targets": "all",
  "externalBin": ["binaries/ifc-tool"],
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ],
  "windows": {
    "nsis": {
      "installMode": "perUser",
      "languages": ["Dutch"],
      "displayLanguageSelector": false,
      "installerIcon": "icons/icon.ico",
      "shortcutName": "ISSO 51 Warmteverliesberekening"
    }
  }
}
```

- [ ] **Step 2.2: Verifieer JSON-validiteit**

Run:
```powershell
node -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json', 'utf8')); console.log('OK')"
```
Expected: `OK` (geen parse-errors).

- [ ] **Step 2.3: Commit**

```powershell
git add src-tauri/tauri.conf.json
git commit -m "feat(installer): add NSIS config — per-user, NL wizard, shortcut name"
```

---

## Task 3: Versie-sync script (`tools/sync-version.ps1`)

**Files:**
- Create: `tools/sync-version.ps1`

- [ ] **Step 3.1: Maak het script**

Maak `tools/sync-version.ps1` met deze inhoud:

```powershell
<#
.SYNOPSIS
Sync workspace version from Cargo.toml to tauri.conf.json and frontend/package.json.

.DESCRIPTION
Single source of truth: Cargo.toml [workspace.package] version.
This script reads it and writes to:
  - src-tauri/tauri.conf.json (top-level "version")
  - frontend/package.json (top-level "version")

Run from repo root.
#>

$ErrorActionPreference = 'Stop'

# Ensure we run from repo root (Cargo.toml workspace must exist here)
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

# 2. Sync tauri.conf.json
$tauriConfPath = 'src-tauri/tauri.conf.json'
$tauriConf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
$tauriConf.version = $version
$tauriConf | ConvertTo-Json -Depth 32 | Set-Content $tauriConfPath -Encoding utf8
Write-Host "Updated $tauriConfPath -> version $version"

# 3. Sync frontend/package.json
$pkgPath = 'frontend/package.json'
$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$pkg.version = $version
$pkg | ConvertTo-Json -Depth 32 | Set-Content $pkgPath -Encoding utf8
Write-Host "Updated $pkgPath -> version $version"

Write-Host "Version sync complete: $version"
```

- [ ] **Step 3.2: Run het script**

```powershell
pwsh -File tools/sync-version.ps1
```
Expected output (versie kan afwijken):
```
Workspace version: 0.1.1
Updated src-tauri/tauri.conf.json -> version 0.1.1
Updated frontend/package.json -> version 0.1.1
Version sync complete: 0.1.1
```

- [ ] **Step 3.3: Verifieer dat versies gelijk zijn**

```powershell
$cargoRaw = Get-Content 'Cargo.toml' -Raw
$null = $cargoRaw -match '(?ms)\[workspace\.package\].*?version\s*=\s*"([^"]+)"'
$cargo = $Matches[1]
$tauri = (Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version
$pkg = (Get-Content frontend/package.json -Raw | ConvertFrom-Json).version
Write-Host "Cargo:    $cargo"
Write-Host "Tauri:    $tauri"
Write-Host "package:  $pkg"
if ($cargo -eq $tauri -and $tauri -eq $pkg) { Write-Host "MATCH" } else { Write-Host "MISMATCH"; exit 1 }
```
Expected: alle drie dezelfde versie + `MATCH`.

- [ ] **Step 3.4: Commit**

```powershell
git add tools/sync-version.ps1 src-tauri/tauri.conf.json frontend/package.json
git commit -m "feat(installer): version-sync script — Cargo.toml as single source of truth"
```

---

## Task 4: Placeholder source-icon

**Files:**
- Create: `tools/make-placeholder-icon.ps1`
- Create (generated): `src-tauri/icons/source.png`

- [ ] **Step 4.1: Maak het generator-script**

Maak `tools/make-placeholder-icon.ps1`:

```powershell
<#
.SYNOPSIS
Generate a 1024x1024 placeholder PNG for the app icon.

.DESCRIPTION
Creates a flat blue square with "I51" centered in white.
Run once; commit the resulting src-tauri/icons/source.png.
Replace with real branding when available.
#>

$ErrorActionPreference = 'Stop'

if (-not (Test-Path 'src-tauri/icons')) {
    New-Item -ItemType Directory -Path 'src-tauri/icons' | Out-Null
}

Add-Type -AssemblyName System.Drawing

$size = 1024
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias

# Background: ISSO-blue
$bgColor = [System.Drawing.Color]::FromArgb(255, 26, 79, 122)
$g.Clear($bgColor)

# Text "I51" centered, white, bold Segoe UI
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$font = New-Object System.Drawing.Font 'Segoe UI', 280, ([System.Drawing.FontStyle]::Bold)
$format = New-Object System.Drawing.StringFormat
$format.Alignment = [System.Drawing.StringAlignment]::Center
$format.LineAlignment = [System.Drawing.StringAlignment]::Center
$rect = New-Object System.Drawing.RectangleF 0, 0, $size, $size
$g.DrawString('I51', $font, $brush, $rect, $format)

$outPath = 'src-tauri/icons/source.png'
$bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)

$g.Dispose()
$bmp.Dispose()
$brush.Dispose()
$font.Dispose()

Write-Host "Wrote $outPath ($size x $size)"
```

- [ ] **Step 4.2: Run het script**

```powershell
pwsh -File tools/make-placeholder-icon.ps1
```
Expected: `Wrote src-tauri/icons/source.png (1024 x 1024)`.

- [ ] **Step 4.3: Verifieer afmetingen**

```powershell
Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile((Resolve-Path 'src-tauri/icons/source.png'))
Write-Host "$($img.Width) x $($img.Height)"
$img.Dispose()
```
Expected: `1024 x 1024`.

- [ ] **Step 4.4: Commit**

```powershell
git add tools/make-placeholder-icon.ps1 src-tauri/icons/source.png
git commit -m "feat(installer): placeholder source icon (I51 on blue) for icon-set generation"
```

---

## Task 5: Genereer Tauri icon-set

**Files:**
- Create (generated): `src-tauri/icons/icon.ico`, `icon.icns`, `32x32.png`, `128x128.png`, `128x128@2x.png`

- [ ] **Step 5.1: Installeer tauri CLI als nog niet aanwezig**

```powershell
npx --yes @tauri-apps/cli --version
```
Expected: een versie als `2.x.x`. Als de CLI ontbreekt, downloadt npx 'm automatisch.

- [ ] **Step 5.2: Genereer de icon-set**

```powershell
npx @tauri-apps/cli icon src-tauri/icons/source.png --output src-tauri/icons
```
Expected output: regels die aangeven dat icoon-bestanden zijn aangemaakt (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`, en eventueel extra Linux/Android sizes).

- [ ] **Step 5.3: Verifieer dat alle 5 vereiste bestanden bestaan**

```powershell
$required = @('32x32.png', '128x128.png', '128x128@2x.png', 'icon.icns', 'icon.ico')
$missing = $required | Where-Object { -not (Test-Path "src-tauri/icons/$_") }
if ($missing) {
    Write-Host "MISSING: $($missing -join ', ')"
    exit 1
} else {
    Write-Host "All 5 required icons present."
}
```
Expected: `All 5 required icons present.`.

- [ ] **Step 5.4: Commit**

```powershell
git add src-tauri/icons/
git commit -m "feat(installer): generated Tauri icon-set (32/128/256 PNG + icns + ico)"
```

Note: de Tauri CLI kan ook andere bestanden genereren (Android, iOS, Linux). Die mogen ook gecommit worden — geen kwaad.

---

## Task 6: Lokaal build-script (`tools/build-installer.ps1`)

**Files:**
- Create: `tools/build-installer.ps1`

- [ ] **Step 6.1: Maak het script**

Maak `tools/build-installer.ps1`:

```powershell
<#
.SYNOPSIS
Build the Windows NSIS installer for ISSO 51 Warmteverliesberekening.

.DESCRIPTION
Orchestrates the full build:
  1. Verify environment (node, npm, cargo).
  2. Sync version from Cargo.toml workspace.
  3. Install + build frontend.
  4. Run Tauri NSIS bundle.
  5. Copy resulting .exe to dist/installer/.

Run from repo root: pwsh -File tools/build-installer.ps1
#>

$ErrorActionPreference = 'Stop'

# 1. Sanity checks
if (-not (Test-Path 'Cargo.toml')) {
    throw "Cargo.toml not found. Run from repo root."
}

$tools = @('node', 'npm', 'cargo')
foreach ($tool in $tools) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "Required tool not found: $tool. See docs/building-installer.md."
    }
}

# Verify icons exist (Tauri build will fail without them)
$requiredIcons = @(
    'src-tauri/icons/icon.ico',
    'src-tauri/icons/32x32.png',
    'src-tauri/icons/128x128.png',
    'src-tauri/icons/128x128@2x.png'
)
foreach ($icon in $requiredIcons) {
    if (-not (Test-Path $icon)) {
        throw "Required icon missing: $icon. Run tools/make-placeholder-icon.ps1 + npx tauri icon."
    }
}

Write-Host "==> Environment OK" -ForegroundColor Green

# 2. Sync version
Write-Host "==> Syncing version..." -ForegroundColor Cyan
pwsh -File tools/sync-version.ps1
if ($LASTEXITCODE -ne 0) { throw "sync-version.ps1 failed" }

# 3. Frontend build
Write-Host "==> Installing frontend dependencies..." -ForegroundColor Cyan
Push-Location frontend
try {
    npm install
    if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
    Write-Host "==> Building frontend..." -ForegroundColor Cyan
    npm run build
    if ($LASTEXITCODE -ne 0) { throw "frontend build failed" }
} finally {
    Pop-Location
}

# 4. Tauri bundle (NSIS only)
Write-Host "==> Building Tauri NSIS bundle (this can take 5-15 minutes)..." -ForegroundColor Cyan
npx @tauri-apps/cli build --bundles nsis
if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }

# 5. Copy output
$bundleDir = 'src-tauri/target/release/bundle/nsis'
$outputDir = 'dist/installer'
if (-not (Test-Path $bundleDir)) {
    throw "Tauri output dir not found: $bundleDir"
}

if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

$exes = Get-ChildItem -Path $bundleDir -Filter '*.exe'
if ($exes.Count -eq 0) {
    throw "No .exe found in $bundleDir"
}

foreach ($exe in $exes) {
    $dest = Join-Path $outputDir $exe.Name
    Copy-Item $exe.FullName $dest -Force
    Write-Host "==> Output: $dest" -ForegroundColor Green
}

Write-Host ""
Write-Host "Build complete." -ForegroundColor Green
Write-Host "Installer(s) in: $((Resolve-Path $outputDir).Path)"
```

- [ ] **Step 6.2: Run het script — full integration test**

```powershell
pwsh -File tools/build-installer.ps1
```
Expected: het script doorloopt alle stappen, eindigt met `Build complete.` en print het pad naar `dist/installer/ISSO 51 Warmteverliesberekening_<versie>_x64-setup.exe`.

**Dit duurt 5-15 minuten** (eerste Rust build is traag).

Mogelijke fouten:
- `cargo tauri build` faalt op linkfouten → MSVC ontbreekt → installeer Build Tools.
- `npm install` faalt → check Node-versie.
- Sidecar `ifc-tool-x86_64-pc-windows-msvc.exe` ontbreekt → al aanwezig in de repo, maar als die per ongeluk weg is, faalt de bundle. Restore via `git checkout src-tauri/binaries/`.

- [ ] **Step 6.3: Verifieer output**

```powershell
Get-ChildItem dist/installer -Filter '*.exe' | Select-Object Name, Length
```
Expected: één regel met de installer-naam en grootte (~80-200 MB).

- [ ] **Step 6.4: Commit**

```powershell
git add tools/build-installer.ps1
git commit -m "feat(installer): local build script — sync version, build frontend, NSIS bundle"
```

---

## Task 7: Build-documentatie (`docs/building-installer.md`)

**Files:**
- Create: `docs/building-installer.md`

- [ ] **Step 7.1: Maak de documentatie**

Maak `docs/building-installer.md`:

```markdown
# Windows installer bouwen

Korte handleiding om lokaal een `.exe` installer te bouwen voor ISSO 51 Warmteverliesberekening.

## Vereisten

- Windows 10 of 11 (x64)
- [Node.js 22 LTS](https://nodejs.org/)
- [Rust toolchain (stable)](https://rustup.rs/) — minimaal 1.78
- [PowerShell 7+](https://aka.ms/powershell) — `winget install Microsoft.PowerShell`
- Visual Studio Build Tools 2022 met **Desktop development with C++** workload — vereist door Tauri/Rust voor MSVC linker
- Git

## Eerste keer setup

```powershell
git clone https://github.com/OpenAEC-Foundation/open-heatloss-studio
cd open-heatloss-studio
```

## Build

Vanuit de repo-root:
```powershell
pwsh -File tools/build-installer.ps1
```

Het script doet:
1. Sanity-check op `node`, `npm`, `cargo` en de iconen.
2. Sync de versie uit `Cargo.toml` workspace naar `tauri.conf.json` en `frontend/package.json`.
3. `npm install` + `npm run build` in `frontend/`.
4. `npx tauri build --bundles nsis` om alleen de NSIS-installer te bouwen.
5. Kopieert de resulterende `.exe` naar `dist/installer/`.

**Duur:** 5-15 minuten (eerste keer). Daarna 2-5 minuten dankzij Rust incremental builds.

## Output

```
dist/installer/ISSO 51 Warmteverliesberekening_<versie>_x64-setup.exe
```

Run dit `.exe` om te installeren. De wizard is in het Nederlands. Default install-locatie is `%LOCALAPPDATA%\Programs\ISSO 51 Warmteverliesberekening`. Geen admin-rechten nodig.

## Versie wijzigen

Versie staat in `Cargo.toml` workspace:
```toml
[workspace.package]
version = "0.2.0"
```

Het build-script sync't `tauri.conf.json` en `frontend/package.json` automatisch.

## Iconen vervangen

De huidige iconen zijn placeholders ("I51" op blauw). Voor echte branding:

1. Vervang `src-tauri/icons/source.png` met een 1024×1024 PNG (transparante achtergrond aanbevolen).
2. Genereer de icon-set:
   ```powershell
   npx @tauri-apps/cli icon src-tauri/icons/source.png --output src-tauri/icons
   ```
3. Commit de gewijzigde bestanden.

## Troubleshooting

| Probleem | Oplossing |
|---|---|
| `cargo: command not found` | Installeer Rust via https://rustup.rs/ |
| `link.exe not found` | Installeer Visual Studio Build Tools met C++ workload |
| `MSB8003` of MSVC-fouten | Heropen je terminal na install Build Tools (PATH-update) |
| `npm install` faalt op `web-ifc.wasm` | Check dat `frontend/node_modules/web-ifc/web-ifc.wasm` bestaat na install |
| Installer-build duurt > 30 min | Eerste build van Rust dependencies is traag; daarna sneller via cache |
| Sidecar `ifc-tool` ontbreekt | `git checkout src-tauri/binaries/` |
| Anti-virus blokkeert build | Voeg `target/` map toe aan exclusions |

## Niet bij deze build (komt later)

- Code signing (Authenticode certificaat) — installer toont nu SmartScreen-warning bij downloaden.
- Auto-update via Tauri updater plugin — komt in PR 2 (alleen notificatie, geen auto-install).
- macOS / Linux installers — vereist sidecar voor die platforms.
- WiX MSI — komt zodra enterprise-rollout relevant is.
```

- [ ] **Step 7.2: Verifieer markdown**

```powershell
Test-Path docs/building-installer.md
```
Expected: `True`.

- [ ] **Step 7.3: Commit**

```powershell
git add docs/building-installer.md
git commit -m "docs(installer): how to build the Windows installer locally"
```

---

## Task 8: End-to-end installer-test (handmatig)

**Files:** geen wijzigingen — dit is een verificatiestap.

- [ ] **Step 8.1: Run de installer**

Dubbelklik op het `.exe` in `dist/installer/`, of:
```powershell
& "$(Resolve-Path dist/installer/*.exe)"
```

- [ ] **Step 8.2: Verifieer wizard**

Controleer:
- Wizard-tekst is in **het Nederlands** (geen taal-keuze schermpje vooraf).
- Geen UAC-prompt (per-user install).
- Default install-pad: `%LOCALAPPDATA%\Programs\ISSO 51 Warmteverliesberekening` (zichtbaar in een advanced-optie als die er is, of als finished-screen).
- Optie "Snelkoppeling op bureaublad" en "Programma direct starten" zijn aanwezig.

Voltooi de installatie.

- [ ] **Step 8.3: Verifieer install-locatie**

```powershell
Test-Path "$env:LOCALAPPDATA\Programs\ISSO 51 Warmteverliesberekening\isso51-desktop.exe"
```
Expected: `True`.

- [ ] **Step 8.4: Verifieer Start-menu shortcut**

```powershell
Get-ChildItem "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Recurse -Filter '*ISSO*'
```
Expected: een `.lnk` bestand met "ISSO 51 Warmteverliesberekening" in de naam.

- [ ] **Step 8.5: Start de app via shortcut**

Klik de Start-menu shortcut. Verwacht:
- App-window opent zonder OS-decorations (custom TitleBar).
- Frontend laadt zonder errors (geen blanco scherm).
- Console-errors? Open DevTools (F12 in Tauri dev-mode; in release-mode niet mogelijk — check `%LOCALAPPDATA%\Programs\...\` voor crash-logs).

- [ ] **Step 8.6: Test sidecar**

Importeer een IFC-bestand via de UI (als beschikbaar in de Backstage/import-flow). Verwacht: sidecar `ifc-tool` reageert; modeller toont geïmporteerde ruimtes.

Als geen test-IFC voorhanden: open de DevTools console of check de logs of de sidecar überhaupt opstart.

- [ ] **Step 8.7: Test uninstall**

Via Windows Settings → Apps → "ISSO 51 Warmteverliesberekening" → Uninstall.

Verifieer:
```powershell
Test-Path "$env:LOCALAPPDATA\Programs\ISSO 51 Warmteverliesberekening"
```
Expected: `False`.

- [ ] **Step 8.8: (Geen commit) — handmatige verificatie afgerond**

Als alle stappen 8.1-8.7 slagen: PR 1 is klaar voor merge. Geen extra commit nodig.

Als één stap faalt: documenteer wat er misgaat, ga terug naar de relevante Task hierboven.

---

## Final checklist

- [ ] Alle commits aanwezig op de branch (`git log --oneline` toont tasks 1-7).
- [ ] `dist/installer/*.exe` bestaat en installeert succesvol.
- [ ] Wizard in NL, per-user install, shortcut werkt, app start, uninstall werkt.
- [ ] Geen wijzigingen buiten de bestanden gespecificeerd in "File Structure" hierboven (behalve eventuele door `npx tauri icon` gegenereerde extra bestanden zoals Android/Linux/iOS-iconen).
- [ ] PR aanmaken: titel `feat(installer): Windows NSIS installer (lokale build)`, body verwijst naar [docs/superpowers/specs/2026-05-08-windows-installer-design.md](../specs/2026-05-08-windows-installer-design.md) en deze plan.

---

## Wat hierna komt (PR 2 — apart plan)

Pas na merge van PR 1:
1. GitHub Actions workflow (`.github/workflows/release.yml`) — tag-based release naar GitHub Releases.
2. Frontend update-check (`frontend/src/lib/updateCheck.ts`) — vergelijk huidige versie met `releases/latest`.
3. UpdateBanner component in TitleBar.
4. `docs/releasing.md` — release-procedure.

Het plan voor PR 2 wordt geschreven nadat PR 1 gemerged is, omdat dat de exacte integratie-paden voor de UpdateBanner vastlegt.
