# isso51-desktop (Tauri v2)

Desktop-shell van Open Heatloss Studio. Frontend: `../frontend` (React/Vite).

## Filesystem-scope (`capabilities/default.json`)

De webview heeft via `tauri-plugin-fs` alleen toegang binnen de statische
scope hieronder. De vorige scope (`**` = volledig filesystem, audit §2.2
MAJOR) is vervangen door de smalste set die de bestaande flows dekt.

| Scope-entry | Waarvoor nodig |
|---|---|
| `$DOCUMENT/**` | Default save-pad `<Documents>/Open Heatloss Studio/<naam>.ifcenergy` (Bestand → Opslaan zonder bekend pad, incl. `mkdir` van die map — `AppShell.deriveDefaultSavePath`, `Backstage.handleSave`) + norm-wissel back-ups (`normSwitch.deriveBackupPath` fallback) + recent-files/stille saves van projecten in Documenten |
| `$DESKTOP/**` | Recent-files openen / stil terugschrijven / norm-wissel back-up naast projectbestanden op het bureaublad |
| `$DOWNLOAD/**` | Idem voor `.ifcenergy`-bestanden die via de webversie gedownload zijn |

**Geen statische scope nodig voor:**

- **Bestand → Openen** en **Opslaan als…** — de dialog-plugin
  (`tauri-plugin-dialog`) voegt het door de user gekozen pad runtime toe aan
  de fs-scope (`allow_file` in de dialog-commands). Werkt dus overal,
  inclusief netwerkschijven.
- **File-association** (dubbelklik `.ifcenergy` in Explorer) — `lib.rs`
  allowlist het argv-pad runtime via `fs_scope().allow_file()` vóór het
  `open-file` event naar de frontend gaat.

**Bewust buiten scope (gedrag bij paden erbuiten):**

- Recent-bestand op bv. een netwerkschijf na een app-herstart: `readTextFile`
  faalt → bestaande nette fallback in `Backstage.handleOpenRecent` ("kies het
  bestand opnieuw" → open-dialog, die het pad weer runtime allowlist).
- Norm-wissel back-up naast een project buiten de scope: Tauri-write faalt →
  bestaande blob-download-fallback in `normSwitch.writeNormSwitchBackup`.
- Stille save (Bestand → Opslaan) naar een pad buiten de scope dat niet in
  deze sessie via dialog/file-association geopend is: error-toast → user kiest
  "Opslaan als…".

Scope-variabelen (`$DOCUMENT` e.d.) worden door Tauri via de Windows
known-folder API geresolved, dus OneDrive-omgeleide mappen werken mee.

Runtime-extensies (dialog/file-association) gelden per sessie en worden niet
gepersisteerd (geen `tauri-plugin-persisted-scope`).

Het file-association argv-pad wordt vóór het allowlisten gevalideerd in
`launched_with_file()` (`src/lib.rs`): alleen `.ifcenergy`/`.json`-extensies
(case-insensitive) én alleen een bestaand regulier bestand (geen directory).

## Shell-permissions

Alleen `shell:allow-open` (FeedbackDialog opent URLs in de default browser).
`shell:allow-execute` is verwijderd: de webview voert nergens processen uit —
de `ifc-tool` sidecar draait Rust-side via `ShellExt` in `commands.rs` en
heeft daar geen webview-permission voor nodig.
