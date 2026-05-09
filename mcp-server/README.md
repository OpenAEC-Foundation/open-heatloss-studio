# Open Heatloss Studio — MCP Server

MCP (Model Context Protocol) server voor de ISSO 51 warmteverlies-rekenkern.
Hiermee kunnen Claude Desktop, Claude Code en andere MCP-clients direct
projecten openen, berekenen en resultaten inspecteren.

Pattern gespiegeld op
[`open-calc-studio/mcp-server`](https://github.com/OpenAEC-Foundation/open-calc-studio/tree/main/mcp-server).

## Setup

```bash
# 1. Build de Rust calc-engine (eenmalig of bij wijziging in isso51-core)
cd ..               # naar repo-root
cargo build --release -p isso51-core --example calc_from_file

# 2. Installeer Node deps en start MCP server
cd mcp-server
npm install
npm start
```

## Gebruik in Claude Code / Claude Desktop

Voeg aan je MCP-config toe (bv. `~/.claude/mcp_servers.json` of
`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "open-heatloss-studio": {
      "command": "npx",
      "args": ["tsx", "src/index.ts"],
      "cwd": "<absolute-path-to-repo>/mcp-server"
    }
  }
}
```

Of, na `npm run bundle`:

```json
{
  "mcpServers": {
    "open-heatloss-studio": {
      "command": "node",
      "args": ["<absolute-path-to-repo>/mcp-server/dist/ohs-mcp.mjs"]
    }
  }
}
```

## Tools

### `calculate`

Bereken warmteverlies uit een Project JSON-object.

| Parameter | Type | Required | Omschrijving |
|---|---|---|---|
| `project` | object | Ja | Volledig Project JSON (zie `schemas/v1/project.schema.json`) |

Output: ProjectResult JSON met per-room en building summary.

### `calculate_file`

Open een `.ifcenergy` of `.isso51.json` bestand en bereken het project.
Werkt ook met legacy raw Project JSON. Update tegelijk `project://current`
en `result://current`.

| Parameter | Type | Required | Omschrijving |
|---|---|---|---|
| `filePath` | string | Ja | Absoluut pad naar het bestand |

Output: korte samenvatting (aansluitvermogen, transmissie, ventilatie,
systeemverliezen) plus pointer naar `result://current` voor het volledige
resultaat.

### `generate_pdf`

Genereer PDF-rapport uit een project-bestand. Vereist dat het `gen_pdf`
CLI binary is gebouwd (`cargo build --release --bin gen_pdf` vanaf
repo-root, of download `gen-pdf-cli` artifact uit de laatste CI run).

| Parameter | Type | Required | Omschrijving |
|---|---|---|---|
| `inputPath` | string | Ja | Absoluut pad naar .ifcenergy / .isso51.json / project.json |
| `outputPath` | string | Ja | Absoluut pad voor de output PDF |

### `parse_ifcenergy`

Parse een `.ifcenergy` document zonder te berekenen — geeft `{ project,
result, modeller }` envelope terug zoals opgeslagen.

### `get_schema`

Haal het JSON-schema van `project` of `result` op.

| Parameter | Type | Required | Omschrijving |
|---|---|---|---|
| `name` | `"project"` \| `"result"` | Ja | Welk schema |

### `list_constructions`

Lijst van standaard constructies uit de bibliotheek (Rc-waarden, materiaal-lagen).

## Resources

- `project://current` — laatst geladen Project JSON
- `result://current` — laatst berekende ProjectResult JSON

## Hoe het werkt

De MCP server is een TypeScript Node.js proces dat via `child_process`
het `calc_from_file` cargo-example aanroept (in `target/release/examples/`).
Geen Tauri-stack vereist; isso51-core is een puur Rust-crate die op elke
toolchain compileert.

In-memory state houdt het laatst geladen project + resultaat zodat
opeenvolgende tool-calls + resource-reads kunnen samenwerken (bv. eerst
`calculate_file` → daarna `result://current` lezen).

## Niet in deze versie

- Geen modificatie-tools (add_room, update_construction, etc) — alleen-lezen
  inspectie + berekening. Mutaties komen in een latere PR samen met de
  editable-modus van de modeller.
- Geen WebSocket-bridge naar de draaiende Tauri-app (zoals OCS heeft op
  `ws://127.0.0.1:9741`). Mogelijk in een toekomstige PR voor live-syncing.
- Geen IFC-import via sidecar — vereist Tauri runtime.
