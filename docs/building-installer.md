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
