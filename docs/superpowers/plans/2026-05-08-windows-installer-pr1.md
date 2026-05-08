# Windows Installer PR 1 — CI-bouwbare installer (artifact)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Een `.exe` Windows-installer kunnen bouwen via GitHub Actions (`windows-latest` runner) met `workflow_dispatch` trigger, die als build-artifact te downloaden is voor handmatig testen.

**Architecture:** GitHub Actions runner heeft MSVC build tools standaard, dus geen lokale Visual Studio Build Tools nodig. Tauri v2 NSIS-bundler genereert het `.exe`. Versie-sync gebeurt in CI via een PowerShell-script dat ook lokaal draait. Placeholder-icons gegenereerd via System.Drawing in PowerShell — vervangbaar zodra echte branding er is.

**Tech Stack:** Tauri v2, NSIS, GitHub Actions (`windows-latest`), PowerShell (5.1 compatible), Node 22, Rust stable MSVC.

**Spec reference:** [docs/superpowers/specs/2026-05-08-windows-installer-design.md](../specs/2026-05-08-windows-installer-design.md)

**Plan-shift 2026-05-08:** Origineel plan was lokaal-eerst, maar lokale Visual Studio Build Tools ontbreken. CI-first volgorde: PR 1 levert een GitHub Actions workflow die het `.exe` als artifact bouwt; lokaal build-script verschuift naar PR 2.

**Niet in deze PR:** Lokaal `tools/build-installer.ps1` script, release-on-tag automation, frontend update-check, update-banner UI. Komt in PR 2.

---

## File Structure

| Bestand | Status | Verantwoordelijkheid |
|---|---|---|
| `.gitignore` | wijzigen | Negeer build-output `dist/installer/` (voor wanneer lokaal builden in PR 2 wel mogelijk wordt) |
| `src-tauri/tauri.conf.json` | wijzigen | NSIS-config (per-user, NL, shortcut-naam, installer-icon) |
| `tools/sync-version.ps1` | nieuw | Lees versie uit `Cargo.toml` workspace; schrijf naar `tauri.conf.json` + `frontend/package.json` |
| `tools/make-placeholder-icon.ps1` | nieuw | Genereer 1024×1024 placeholder PNG (eenmalig lokaal, op Windows) |
| `src-tauri/icons/source.png` | nieuw (gegenereerd, gecommit) | Bron voor Tauri icon-converter |
| `src-tauri/icons/icon.ico` | nieuw (gegenereerd, gecommit) | Windows installer + executable icon |
| `src-tauri/icons/32x32.png` | nieuw (gegenereerd, gecommit) | Tauri-required size |
| `src-tauri/icons/128x128.png` | nieuw (gegenereerd, gecommit) | Tauri-required size |
| `src-tauri/icons/128x128@2x.png` | nieuw (gegenereerd, gecommit) | Tauri-required size (256×256) |
| `src-tauri/icons/icon.icns` | nieuw (gegenereerd, gecommit) | macOS icon (Tauri verwacht dit ook bij Windows-build) |
| `.github/workflows/build-installer.yml` | nieuw | `workflow_dispatch` trigger, bouwt NSIS `.exe` op windows-latest, upload als artifact |
| `docs/building-installer.md` | nieuw | Hoe trigger je de CI-build, hoe download je het artifact, hoe install/test je het |

---

## Task 1: `.gitignore` update

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1.1: Voeg `dist/installer/` toe**

Open `.gitignore` en voeg toe aan het eind:
```
# Built installers (lokaal in PR 2; CI artifacts staan op GitHub)
dist/installer/
```

- [ ] **Step 1.2: Verifieer**

```powershell
New-Item -ItemType Directory -Force -Path dist/installer | Out-Null
New-Item dist/installer/test.exe -ItemType File | Out-Null
git status --short dist/
Remove-Item -Recurse -Force dist
```
Expected: geen output van `git status --short dist/` (de map wordt genegeerd).

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

Huidige `bundle`-sectie in `src-tauri/tauri.conf.json`:
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

```powershell
node -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json', 'utf8')); console.log('OK')"
```
Expected: `OK`.

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

Maak `tools/sync-version.ps1`. Belangrijk: gebruik `[System.IO.File]::WriteAllText` met UTF-8 zonder BOM zodat het werkt in zowel PowerShell 5.1 (Windows default) als 7+.

```powershell
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
```

- [ ] **Step 3.2: Run het script**

```powershell
powershell -ExecutionPolicy Bypass -File tools/sync-version.ps1
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
Run once on Windows; commit the resulting src-tauri/icons/source.png.
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
powershell -ExecutionPolicy Bypass -File tools/make-placeholder-icon.ps1
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

- [ ] **Step 5.1: Genereer de icon-set met de Tauri CLI**

```powershell
npx --yes @tauri-apps/cli icon src-tauri/icons/source.png --output src-tauri/icons
```
Expected output: regels die aangeven dat icoon-bestanden zijn aangemaakt (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`, en mogelijk extra Linux/Android sizes).

Geen Rust build nodig — `tauri icon` is een Node-only command.

- [ ] **Step 5.2: Verifieer dat alle 5 vereiste bestanden bestaan**

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

- [ ] **Step 5.3: Commit**

```powershell
git add src-tauri/icons/
git commit -m "feat(installer): generated Tauri icon-set (32/128/256 PNG + icns + ico)"
```

Note: de Tauri CLI genereert mogelijk ook Android/iOS/Linux varianten — die mogen ook gecommit worden.

---

## Task 6: GitHub Actions workflow (`build-installer.yml`)

**Files:**
- Create: `.github/workflows/build-installer.yml`

- [ ] **Step 6.1: Maak de workflow**

Maak `.github/workflows/build-installer.yml`:

```yaml
name: Build Windows installer

on:
  workflow_dispatch:
    inputs:
      reason:
        description: "Reason for manual run (optional, shown in run name)"
        required: false
        default: "manual test build"

run-name: "Installer build — ${{ inputs.reason }}"

permissions:
  contents: read

concurrency:
  group: build-installer-${{ github.ref }}
  cancel-in-progress: true

jobs:
  build:
    runs-on: windows-latest
    timeout-minutes: 30

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node.js 22
        uses: actions/setup-node@v4
        with:
          node-version: "22"
          cache: "npm"
          cache-dependency-path: frontend/package-lock.json

      - name: Setup Rust (stable, MSVC)
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable-x86_64-pc-windows-msvc

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            src-tauri/target
          key: cargo-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}
          restore-keys: |
            cargo-${{ runner.os }}-

      - name: Sync version
        shell: pwsh
        run: ./tools/sync-version.ps1

      - name: Install frontend deps
        working-directory: frontend
        run: npm ci

      - name: Build NSIS installer
        run: npx --yes @tauri-apps/cli build --bundles nsis

      - name: Locate built installer
        id: locate
        shell: pwsh
        run: |
          $exe = Get-ChildItem -Path src-tauri/target/release/bundle/nsis -Filter '*.exe' | Select-Object -First 1
          if (-not $exe) { throw "No .exe found in src-tauri/target/release/bundle/nsis" }
          "exe_path=$($exe.FullName)" | Out-File -FilePath $env:GITHUB_OUTPUT -Append
          "exe_name=$($exe.Name)" | Out-File -FilePath $env:GITHUB_OUTPUT -Append
          Write-Host "Found: $($exe.FullName)"

      - name: Upload installer as artifact
        uses: actions/upload-artifact@v4
        with:
          name: windows-installer
          path: ${{ steps.locate.outputs.exe_path }}
          retention-days: 14
          if-no-files-found: error
```

- [ ] **Step 6.2: Verifieer YAML-syntax**

```powershell
# Quick check by Node — no extra dep needed (just JSON-via-YAML conversion not needed; we just check it's parseable text)
$yaml = Get-Content .github/workflows/build-installer.yml -Raw
if ($yaml -match 'workflow_dispatch' -and $yaml -match 'windows-latest' -and $yaml -match 'upload-artifact') {
    Write-Host "Workflow contains expected keywords."
} else {
    Write-Host "Workflow missing expected keywords."; exit 1
}
```
Expected: `Workflow contains expected keywords.`.

(Echt YAML-parsen kan via een actie als `actionlint`, maar die staat hier niet beschikbaar. GitHub valideert de YAML zodra de workflow gepusht is.)

- [ ] **Step 6.3: Commit**

```powershell
git add .github/workflows/build-installer.yml
git commit -m "ci(installer): GitHub Actions workflow — build NSIS installer on windows-latest, upload as artifact"
```

---

## Task 7: Build-documentatie (`docs/building-installer.md`)

**Files:**
- Create: `docs/building-installer.md`

- [ ] **Step 7.1: Maak de documentatie**

Maak `docs/building-installer.md`:

```markdown
# Windows installer bouwen

De Windows `.exe` installer wordt gebouwd via **GitHub Actions** (geen lokale Visual Studio Build Tools nodig).

## Snel: bouw + download

1. Ga naar de repo op GitHub: https://github.com/OpenAEC-Foundation/open-heatloss-studio
2. Klik op **Actions** → **Build Windows installer** (in de linkerlijst).
3. Klik rechtsboven op **Run workflow** → kies branch (meestal `master` of de feature branch) → optioneel een reden invullen → **Run workflow**.
4. Wacht ~10-15 minuten.
5. Open de geslaagde run, scroll naar **Artifacts** onderaan, download `windows-installer`.
6. Pak de zip uit. Daarin staat het `.exe` bestand.

Of via de CLI:
```powershell
gh workflow run build-installer.yml -R OpenAEC-Foundation/open-heatloss-studio
gh run list --workflow=build-installer.yml --limit 1
gh run download <run-id> --name windows-installer
```

## Wat het `.exe` doet

- Wizard in het Nederlands (geen taalkeuze vooraf).
- **Per-user installatie** in `%LOCALAPPDATA%\Programs\ISSO 51 Warmteverliesberekening`.
- Geen UAC-prompt, geen admin-rechten nodig.
- Maakt Start-menu shortcut "ISSO 51 Warmteverliesberekening".
- SmartScreen-waarschuwing verschijnt (niet code-signed) — klik "Meer info" → "Toch uitvoeren".

## Versie wijzigen

Versie staat in `Cargo.toml` workspace:
```toml
[workspace.package]
version = "0.2.0"
```

Bij elke CI-build sync't `tools/sync-version.ps1` deze waarde naar `tauri.conf.json` en `frontend/package.json`. Daarna verschijnt de versie in de installer-naam: `ISSO 51 Warmteverliesberekening_0.2.0_x64-setup.exe`.

## Iconen vervangen (placeholder → echte branding)

Huidige iconen zijn placeholders ("I51" op blauw). Voor echte branding:

1. Vervang `src-tauri/icons/source.png` met een 1024×1024 PNG (transparante achtergrond aanbevolen).
2. Genereer de icon-set:
   ```powershell
   npx @tauri-apps/cli icon src-tauri/icons/source.png --output src-tauri/icons
   ```
3. Commit + push. Volgende CI-build gebruikt de nieuwe iconen.

## Lokaal bouwen — komt later

Lokaal bouwen vereist:
- Visual Studio Build Tools 2022 met "Desktop development with C++" workload (~5 GB)
- Rust default toolchain switchen naar MSVC: `rustup default stable-x86_64-pc-windows-msvc`

Een lokaal build-script (`tools/build-installer.ps1`) komt in PR 2, voor wie regelmatig snelle dev-builds wil maken zonder GitHub Actions.

## Troubleshooting

| Probleem | Oplossing |
|---|---|
| Workflow faalt op "Setup Rust" | Tijdelijke registry-issue, herhaal de run |
| Workflow faalt op `tauri build` | Check de log; meestal Rust-compile error door verouderde dependency |
| Artifact niet te downloaden | Run moet **succesvol** zijn afgerond (groene vink); failed runs hebben geen artifact |
| Sidecar `ifc-tool` ontbreekt na install | Build-bestand `src-tauri/binaries/ifc-tool-x86_64-pc-windows-msvc.exe` is gecommit; check dat deze in de repo staat |
| SmartScreen blokkeert installer | Niet-gesigneerde installer; klik "Meer info" → "Toch uitvoeren". Code-signing komt zodra cert beschikbaar is |
```

- [ ] **Step 7.2: Verifieer**

```powershell
Test-Path docs/building-installer.md
```
Expected: `True`.

- [ ] **Step 7.3: Commit**

```powershell
git add docs/building-installer.md
git commit -m "docs(installer): how to trigger CI build and download Windows installer artifact"
```

---

## Task 8: Push branch + trigger CI build (handover naar gebruiker)

**Files:** geen wijzigingen — dit is een verificatiestap.

- [ ] **Step 8.1: Push de branch naar GitHub**

```powershell
git push -u origin claude/laughing-kirch-752da4
```
Expected: branch verschijnt op GitHub.

- [ ] **Step 8.2: Trigger de workflow**

Via UI: GitHub → Actions → "Build Windows installer" → Run workflow → kies branch `claude/laughing-kirch-752da4`.

Of via CLI:
```powershell
gh workflow run build-installer.yml --ref claude/laughing-kirch-752da4
```

- [ ] **Step 8.3: Volg de run**

```powershell
gh run watch
```
Expected: build duurt 10-15 minuten, eindigt met groene vink.

Als de run faalt:
1. Check welke step faalt: `gh run view --log-failed`.
2. Diagnosticeer (build-error, timeout, missing file).
3. Fix lokaal, commit, push, trigger opnieuw.

- [ ] **Step 8.4: Download het artifact**

```powershell
$run = gh run list --workflow=build-installer.yml --branch claude/laughing-kirch-752da4 --limit 1 --json databaseId --jq '.[0].databaseId'
gh run download $run --name windows-installer --dir dist/installer
Get-ChildItem dist/installer -Filter '*.exe'
```
Expected: één `.exe` bestand in `dist/installer/`.

- [ ] **Step 8.5: Run de installer (handmatig)**

Dubbelklik op het `.exe` of:
```powershell
& "$(Resolve-Path dist/installer/*.exe)"
```

Verifieer:
- Wizard-tekst is in **het Nederlands**.
- Geen UAC-prompt.
- Default install-pad `%LOCALAPPDATA%\Programs\ISSO 51 Warmteverliesberekening`.
- Optie "Snelkoppeling op bureaublad" en "Programma direct starten" aanwezig.

Voltooi de installatie.

- [ ] **Step 8.6: Verifieer install + start**

```powershell
Test-Path "$env:LOCALAPPDATA\Programs\ISSO 51 Warmteverliesberekening\isso51-desktop.exe"
Get-ChildItem "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Recurse -Filter '*ISSO*'
```
Expected: `True` voor de exe, en een `.lnk` met "ISSO" in de naam.

Start de app via shortcut. Verwacht: window opent zonder OS-decorations (custom TitleBar), frontend laadt.

- [ ] **Step 8.7: Test uninstall**

Windows Settings → Apps → "ISSO 51 Warmteverliesberekening" → Uninstall.

```powershell
Test-Path "$env:LOCALAPPDATA\Programs\ISSO 51 Warmteverliesberekening"
```
Expected: `False`.

- [ ] **Step 8.8: (Geen commit) — handmatige verificatie afgerond**

Als alle stappen 8.5-8.7 slagen: PR 1 is klaar voor merge.

---

## Final checklist

- [ ] Alle commits aanwezig op de branch (`git log --oneline` toont tasks 1-7).
- [ ] CI workflow run is succesvol op de branch.
- [ ] Artifact `windows-installer.zip` bevat een `.exe` dat installeert + start + uninstalled correct.
- [ ] PR aanmaken: titel `feat(installer): Windows NSIS installer via GitHub Actions`, body verwijst naar [docs/superpowers/specs/2026-05-08-windows-installer-design.md](../specs/2026-05-08-windows-installer-design.md) en deze plan.

---

## Wat hierna komt (PR 2 — apart plan)

Pas na merge van PR 1:
1. **Lokaal build-script** (`tools/build-installer.ps1`) — voor dev-iteratie zonder CI-wachttijd (vereist VS Build Tools).
2. **Release-on-tag workflow** — push tag `v*` → automatisch GitHub Release met `.exe` als asset.
3. **Frontend update-check** (`frontend/src/lib/updateCheck.ts`) — vergelijk huidige versie met `releases/latest`.
4. **UpdateBanner** in TitleBar.
5. **`docs/releasing.md`** — release-procedure.

Het plan voor PR 2 wordt geschreven nadat PR 1 gemerged is.
