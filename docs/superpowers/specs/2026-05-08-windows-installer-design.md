# Windows installer voor ISSO 51 Warmteverliesberekening

**Datum:** 2026-05-08
**Status:** Design — klaar voor implementatie

## Doel

Een Windows installer (`.exe`) opleveren waarmee eindgebruikers de Tauri desktop-app kunnen installeren. Distributie via GitHub Releases. Lokaal én via CI bouwbaar. App waarschuwt gebruikers wanneer een nieuwere versie beschikbaar is.

## Beslissingen (vastgelegd in brainstorm)

| Onderwerp | Keuze | Reden |
|---|---|---|
| Platform | Windows | Sidecar `ifc-tool` is alleen voor `x86_64-pc-windows-msvc` gebouwd |
| Format | NSIS `.exe` | Default in Tauri v2, kleiner, per-user install zonder admin |
| Code signing | Nee (voor nu) | Geen Authenticode-cert beschikbaar; SmartScreen-warning geaccepteerd |
| Auto-update | Notificatie alleen | App checkt versie, toont banner, gebruiker downloadt zelf |
| Distributie | GitHub Releases | Repo is publiek; gratis hosting; CORS-vriendelijke API |
| Build-locatie | Lokaal + CI | Lokaal voor dev, CI voor releases |
| Icons | Placeholder | Echte branding komt later; mag nu de build niet blokkeren |
| Install-mode | Per-user (`%LOCALAPPDATA%`) | Geen UAC-prompt, geen admin nodig |
| Wizard-taal | Nederlands | Doelgroep is NL-sprekend (ISSO 51 is een NL-norm) |

## Aanpak: twee PR's

**PR 1 — "Lokaal werkende installer".** Vandaag een `.exe` kunnen bouwen op een Windows-machine.
**PR 2 — "Automatisering + update-check".** Tag → automatische release; app waarschuwt bij nieuwe versie.

PR 2 leunt op PR 1.

---

## PR 1 — concrete deliverables

### 1.1 Versie-synchronisatie

**Probleem:** workspace `Cargo.toml` staat op `0.1.1`, `tauri.conf.json` op `0.1.0`. Twee bronnen die uit sync raken.

**Oplossing:** één bron-of-truth = `Cargo.toml` workspace. Een script `tools/sync-version.ps1` leest `version` uit `Cargo.toml`, schrijft die naar `tauri.conf.json` (`version` veld) en `frontend/package.json` (`version` veld) vóór elke build.

Het script wordt aangeroepen door `tools/build-installer.ps1` en door de CI workflow.

### 1.2 Icons (placeholder)

**Probleem:** `src-tauri/icons/` is leeg. Tauri build faalt zonder.

**Oplossing:** één bron-PNG in `src-tauri/icons/source.png` (1024×1024, simpele placeholder — een gestileerd huis-icoon met de letters "I51" of vergelijkbaar; ik teken die als SVG, render naar PNG).

Dan eenmalig draaien:
```powershell
npm run tauri icon src-tauri/icons/source.png
```
Dit genereert `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico` in `src-tauri/icons/`.

Alle 5 bestanden worden gecommit. De `source.png` ook (zodat regenereren reproduceerbaar is).

**Out of scope:** echte branding/design. Komt later als apart traject.

### 1.3 NSIS-configuratie in `tauri.conf.json`

Toevoegen aan `bundle`:
```json
"windows": {
  "nsis": {
    "installMode": "perUser",
    "languages": ["Dutch"],
    "displayLanguageSelector": false,
    "installerIcon": "icons/icon.ico",
    "shortcutName": "ISSO 51 Warmteverliesberekening"
  }
}
```

`headerImage` en `sidebarImage` (custom branding-afbeeldingen voor de wizard) zijn bewust weggelaten in PR 1 — die bestanden zijn er niet en NSIS gebruikt nette defaults zonder. Toevoegen zodra er echte branding is.

### 1.4 Lokaal build-script: `tools/build-installer.ps1`

Stappen:
1. Sanity-check: zit ik in de repo-root? Is `npm` aanwezig? Is `cargo` aanwezig? Zo niet — stop met duidelijke fout.
2. Roep `tools/sync-version.ps1` aan.
3. `cd frontend && npm install && npm run build` (frontend bouwen).
4. `cd src-tauri && cargo tauri build --bundles nsis` — alleen NSIS, geen MSI/DMG/etc.
5. Kopieer output van `src-tauri/target/release/bundle/nsis/*.exe` naar `dist/installer/` in de repo-root.
6. Print het volledige pad naar de `.exe` als laatste regel.

Het script gebruikt `$ErrorActionPreference = 'Stop'` zodat elke fout direct stopt.

### 1.5 Documentatie: `docs/building-installer.md`

Korte handleiding met:
- Vereisten: Windows 10/11, Node.js ≥ 22, Rust toolchain (1.78+), Visual Studio Build Tools (C++ workload)
- Eerste keer setup: `cargo install tauri-cli --version "^2"` (eenmalig globaal of via `npm install -g`)
- Hoe te draaien: `pwsh -File tools/build-installer.ps1`
- Waar de output staat: `dist/installer/`
- Troubleshooting: `cargo: command not found` → installeer Rust; `MSVC not found` → installeer Build Tools; etc.

---

## PR 2 — concrete deliverables

### 2.1 GitHub Actions workflow: `.github/workflows/release.yml`

**Trigger:** push van een tag `v*` (bv. `v0.2.0`).

**Runner:** `windows-latest`.

**Stappen:**
1. Checkout code.
2. Setup Node 22 + Rust stable + cache (`actions/cache` voor Cargo + npm).
3. Run `tools/sync-version.ps1` (zelfde script als lokaal).
4. `npm install` in `frontend/`, `npm run build`.
5. `cargo tauri build --bundles nsis`.
6. Maak een GitHub Release aan (via `softprops/action-gh-release` of `gh release create`) met:
   - Tag = de pushed tag.
   - Naam = `Versie X.Y.Z`.
   - Body = inhoud van `CHANGELOG.md` (sectie voor deze versie) of een placeholder als die ontbreekt.
   - Assets = de gegenereerde `.exe` uit `src-tauri/target/release/bundle/nsis/`.

**Permissions:** `contents: write` (voor release-create).

**Concurrency:** `group: release-${{ github.ref }}` om dubbele runs op dezelfde tag te voorkomen.

### 2.2 Update-check in de frontend

**Bestand:** `frontend/src/lib/updateCheck.ts`.

**Wat het doet:**
1. Bij app-startup (alleen in Tauri-mode, niet web — check via `import.meta.env.TAURI_PLATFORM` of `window.__TAURI__`).
2. Lees huidige versie uit `package.json` via Vite-define (`import.meta.env.VITE_APP_VERSION`).
3. Fetch `https://api.github.com/repos/OpenAEC-Foundation/open-heatloss-studio/releases/latest`.
4. Vergelijk `tag_name` (gestript van leading `v`) met huidige versie via semver-compare.
5. Als nieuwer → return `{ hasUpdate: true, latestVersion, releaseUrl }`. Anders `{ hasUpdate: false }`.
6. Errors (offline, rate-limit, 404) → return `{ hasUpdate: false }`, geen UI-storing. Log naar console.

**Caching:** sla resultaat op in `localStorage` met timestamp; opnieuw checken max 1× per 6 uur. Voorkomt rate-limiting (GitHub API: 60 req/uur per IP zonder auth).

### 2.3 UI: update-banner

**Bestand:** `frontend/src/components/UpdateBanner.tsx`.

Component die:
- `useEffect` aanroept `checkForUpdates()` uit 2.2.
- Bij `hasUpdate: true` toont een dismissible banner bovenaan in de TitleBar (rechts naast de window-controls). Subtiel, niet-blokkerend.
- Tekst: "Versie X.Y.Z is beschikbaar. [Downloaden]" — link opent `releaseUrl` in default browser via Tauri shell plugin.
- Dismissed-state in `localStorage` per versie (zodat dezelfde update-melding niet eindeloos terugkomt).

Mounten in de root component (`App.tsx`).

### 2.4 Documentatie: `docs/releasing.md`

Release-procedure:
1. Update `CHANGELOG.md` met de wijzigingen voor versie X.Y.Z.
2. Update `Cargo.toml` workspace `version = "X.Y.Z"`.
3. Commit: `chore: bump version to X.Y.Z`.
4. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
5. Wacht op GitHub Actions; controleer dat de release verschijnt op de releases-pagina met `.exe` als asset.

---

## Architectuur en data-flow

### Build-flow (lokaal)
```
Cargo.toml (versie) ──► sync-version.ps1 ──┐
                                            ├──► tauri.conf.json + package.json
                                            └──► (gesynced)
              frontend/ ──► npm run build ──► dist/
                                                │
              src-tauri/ ◄──────────────────────┘
                  │
                  └──► cargo tauri build --bundles nsis
                                │
                                └──► target/release/bundle/nsis/*.exe
                                                  │
                                                  └──► copy ──► dist/installer/*.exe
```

### Release-flow (CI)
```
git tag v0.2.0 ──► push ──► .github/workflows/release.yml
                                │
                                ├──► setup, sync-version, build (zelfde stappen)
                                └──► gh release create v0.2.0 --files *.exe
                                                                 │
                                                                 └──► GitHub Releases
```

### Update-check flow (runtime)
```
App startup ──► updateCheck.ts ──► localStorage (laatst gecheckt < 6u?)
                                        │
                                  ja ──┴──► return cached
                                  nee ──► fetch GitHub API /releases/latest
                                              │
                                              └──► compare versions ──► UpdateBanner
```

## Error handling

| Scenario | Wat gebeurt er |
|---|---|
| Build faalt lokaal door ontbrekende Rust | Script print "cargo niet gevonden — installeer via https://rustup.rs" en stopt |
| Build faalt door ontbrekende icons | Script doet sanity-check op `src-tauri/icons/icon.ico`; print actie als ontbrekend |
| `sync-version.ps1` faalt op JSON-parse | Script stopt; gebruiker moet `tauri.conf.json` valid maken |
| CI build faalt | GitHub Actions toont fout; geen release wordt aangemaakt |
| Update-check: offline | Stilletjes falen, geen banner, console-log |
| Update-check: rate-limit (60/uur) | Stilletjes falen, retry over 6u |
| Update-check: 404 (geen releases yet) | Stilletjes falen, geen banner |

## Testing

**Wat testen we:**
1. **Lokale build slaagt.** Draai `tools/build-installer.ps1` op de huidige Windows machine. Verwacht: `.exe` in `dist/installer/` zonder errors.
2. **Installer installeert correct.** Draai het `.exe`. Wizard in NL. Default install-pad in `%LOCALAPPDATA%\Programs\ISSO 51 Warmteverliesberekening`. Geen UAC-prompt. Start menu shortcut aanwezig.
3. **App start.** Open via shortcut. Frontend laadt. Sidecar `ifc-tool` reageert op een test-IFC import.
4. **Uninstall werkt.** Via Add/Remove Programs verdwijnt alles inclusief shortcut.
5. **CI build slaagt.** Push een test-tag (`v0.0.1-test`) naar een test-branch; verwacht release verschijnt met `.exe`.
6. **Update-check werkt.** Bouw lokaal versie `0.0.1`. Maak handmatig een release `v0.0.2-test` op GitHub. Start app. Verwacht banner "Versie 0.0.2-test is beschikbaar" binnen 5 sec na startup.
7. **Update-check faalt netjes.** Disable internet. App start zonder banner en zonder errors.

**Wat we niet testen (out of scope):**
- Cross-platform (geen macOS/Linux build).
- Auto-update install (alleen notificatie).
- Code signing (geen cert).

## Versionering & release-cadans

- Versie volgt semver vanuit `Cargo.toml` workspace.
- Eerste release na deze PR's: `v0.2.0` — desktop installer is een nieuwe feature die een minor bump rechtvaardigt.
- Geen release vóór PR 1 + PR 2 beide gemerged en getest.

## Wat NIET in deze PR's

- Echte branding/icons (placeholder is fine voor v0.x)
- Code signing (komt zodra cert beschikbaar)
- macOS/Linux installers (sidecar moet eerst cross-platform gebouwd)
- WiX MSI naast NSIS (toevoegen wanneer enterprise-rollout relevant)
- Auto-install van updates (alleen notificatie nu)
- Localization van update-banner naar EN (NL-only voor v0.x, sluit aan bij wizard-taal)

## Bestanden die wijzigen

### PR 1
- **Nieuw:** `src-tauri/icons/source.png`, `icon.ico`, `icon.icns`, `32x32.png`, `128x128.png`, `128x128@2x.png`
- **Nieuw:** `tools/sync-version.ps1`
- **Nieuw:** `tools/build-installer.ps1`
- **Nieuw:** `docs/building-installer.md`
- **Wijzig:** `src-tauri/tauri.conf.json` — versie sync + NSIS-config
- **Wijzig:** `.gitignore` — voeg `dist/installer/` toe

### PR 2
- **Nieuw:** `.github/workflows/release.yml`
- **Nieuw:** `frontend/src/lib/updateCheck.ts`
- **Nieuw:** `frontend/src/components/UpdateBanner.tsx`
- **Nieuw:** `docs/releasing.md`
- **Wijzig:** `frontend/src/App.tsx` (of root layout) — mount `UpdateBanner`
- **Wijzig:** `frontend/vite.config.ts` — define `VITE_APP_VERSION` uit `package.json`
