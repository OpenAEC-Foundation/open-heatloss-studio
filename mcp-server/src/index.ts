#!/usr/bin/env node
/**
 * Open Heatloss Studio MCP Server
 *
 * Stelt de ISSO 51 warmteverlies-rekenkern beschikbaar via Model Context
 * Protocol — Claude Desktop, Claude Code en andere MCP-clients kunnen
 * project-bestanden openen, berekenen en inspecteren.
 *
 * Tools:
 * - calculate         : run warmteverliesberekening op een Project JSON
 * - calculate_file    : open .ifcenergy / .isso51.json en bereken
 * - parse_ifcenergy   : parseer .ifcenergy bestand naar { project, result, modeller }
 * - get_schema        : haal JSON schema (project / result) op
 * - list_constructions: standaard constructies-catalogus
 *
 * Resources:
 * - project://current : laatst geopende project
 * - result://current  : laatst berekende resultaat
 *
 * Pattern gespiegeld op Open Calc Studio's mcp-server/. Gebruikt het
 * `calc_from_file` cargo example als reken-engine (geen runtime Rust
 * nodig in de MCP host — wel een gebouwde binary).
 */
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { execFile } from "node:child_process";
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const REPO_ROOT = resolve(__dirname, "..", "..");

// ---------------------------------------------------------------------------
// In-memory state — laatst geladen project + result als resources
// ---------------------------------------------------------------------------

interface SessionState {
  projectPath: string | null;
  project: unknown | null;
  result: unknown | null;
  modeller: unknown | null;
}

const state: SessionState = {
  projectPath: null,
  project: null,
  result: null,
  modeller: null,
};

// ---------------------------------------------------------------------------
// Calc engine — shells out to the cargo example
// ---------------------------------------------------------------------------

/**
 * Resolve path naar het `calc_from_file` voorbeeld-binary.
 * Build voorwaarde: `cargo build --release -p isso51-core --example calc_from_file`
 * vanaf de repo-root.
 */
function calcBinaryPath(): string {
  // Verwacht onder target/release/examples/calc_from_file(.exe)
  const exeName = process.platform === "win32" ? "calc_from_file.exe" : "calc_from_file";
  const candidates = [
    join(REPO_ROOT, "target", "release", "examples", exeName),
    join(REPO_ROOT, "target", "debug", "examples", exeName),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  throw new Error(
    `calc_from_file binary niet gevonden. Build eerst:\n  cargo build --release -p isso51-core --example calc_from_file\n` +
      `Gezocht in:\n${candidates.map((c) => `  ${c}`).join("\n")}`,
  );
}

/**
 * Bereken Project JSON via het CLI binary. Schrijft de input naar een tijdelijk
 * bestand (binary leest envelope + raw Project beide) en parseert de stdout.
 */
async function runCalc(projectJson: unknown): Promise<unknown> {
  const tmpFile = join(REPO_ROOT, "target", `mcp-input-${Date.now()}.json`);
  writeFileSync(tmpFile, JSON.stringify({ project: projectJson }), "utf8");
  try {
    const bin = calcBinaryPath();
    const { stdout } = await execFileAsync(bin, [tmpFile], {
      maxBuffer: 64 * 1024 * 1024, // 64MB — sommige resultaten zijn groot
    });
    // Het example dumpt eerst "=== Full result JSON ===" daarna pretty JSON.
    // Pak de JSON tussen die marker en de "=== Summary ===" marker.
    const startMarker = "=== Full result JSON ===";
    const endMarker = "=== Summary ===";
    const startIdx = stdout.indexOf(startMarker);
    const endIdx = stdout.indexOf(endMarker);
    if (startIdx < 0 || endIdx < 0) {
      throw new Error(`Onverwachte stdout van calc_from_file:\n${stdout}`);
    }
    const jsonText = stdout.substring(startIdx + startMarker.length, endIdx).trim();
    return JSON.parse(jsonText);
  } finally {
    try {
      writeFileSync(tmpFile, "");
    } catch {
      // negeer
    }
  }
}

// ---------------------------------------------------------------------------
// MCP server
// ---------------------------------------------------------------------------

const server = new McpServer({
  name: "open-heatloss-studio",
  version: "0.1.0",
});

// ----- Tool: calculate ------------------------------------------------------

server.tool(
  "calculate",
  "Bereken warmteverlies (ISSO 51:2023) van een Project JSON. Geeft de full ProjectResult terug — per-room phi_HL/phi_T/phi_V plus building summary (connection_capacity, total_envelope_loss, etc).",
  {
    project: z
      .record(z.unknown())
      .describe("Volledig Project JSON object (zie schemas/v1/project.schema.json)."),
  },
  async ({ project }) => {
    const result = await runCalc(project);
    state.project = project;
    state.result = result;
    return {
      content: [
        { type: "text", text: JSON.stringify(result, null, 2) },
      ],
    };
  },
);

// ----- Tool: calculate_file -------------------------------------------------

server.tool(
  "calculate_file",
  "Open een .ifcenergy of .isso51.json bestand en bereken het project. Update ook de project://current en result://current resources.",
  {
    filePath: z.string().describe("Absoluut pad naar het bestand"),
  },
  async ({ filePath }) => {
    const content = readFileSync(filePath, "utf8");
    const parsed = JSON.parse(content);

    // Detect format: ifcenergy IFCX vs isso51-legacy envelope vs raw
    let project: unknown;
    if (parsed.header?.ifcxVersion && Array.isArray(parsed.data)) {
      // .ifcenergy: vind de envelope
      let envelope: { project: unknown; result?: unknown; modeller?: unknown } | null = null;
      for (const entry of parsed.data) {
        const env = entry.attributes?.["isso51::envelope::v1"];
        if (env) {
          envelope = env;
          break;
        }
      }
      if (!envelope) {
        throw new Error("Geen isso51::envelope::v1 in IFCX document");
      }
      project = envelope.project;
      state.modeller = envelope.modeller ?? null;
    } else if (parsed.schema === "isso51-project-v1" && parsed.project) {
      project = parsed.project;
      state.modeller = parsed.modeller ?? null;
    } else {
      project = parsed;
    }

    const result = await runCalc(project);
    state.projectPath = filePath;
    state.project = project;
    state.result = result;

    const summary = (result as { summary?: unknown })?.summary;
    return {
      content: [
        {
          type: "text",
          text:
            `Berekend uit ${filePath}\n\n` +
            `Connection capacity: ${(summary as { connection_capacity?: number })?.connection_capacity?.toFixed(1) ?? "?"} W\n` +
            `Transmissieverlies:  ${(summary as { total_envelope_loss?: number })?.total_envelope_loss?.toFixed(1) ?? "?"} W\n` +
            `Ventilatieverlies:   ${(summary as { total_ventilation_loss?: number })?.total_ventilation_loss?.toFixed(1) ?? "?"} W\n` +
            `Systeemverliezen:    ${(summary as { total_system_losses?: number })?.total_system_losses?.toFixed(1) ?? "?"} W\n\n` +
            `Volledig resultaat opvraagbaar via resource result://current.`,
        },
      ],
    };
  },
);

// ----- Tool: parse_ifcenergy ------------------------------------------------

server.tool(
  "parse_ifcenergy",
  "Parse een .ifcenergy IFCX bestand zonder te berekenen. Geeft project + result (indien aanwezig) + modeller-snapshot terug.",
  {
    filePath: z.string().describe("Absoluut pad naar het .ifcenergy bestand"),
  },
  async ({ filePath }) => {
    const content = readFileSync(filePath, "utf8");
    const parsed = JSON.parse(content);
    if (!parsed.header?.ifcxVersion || !Array.isArray(parsed.data)) {
      throw new Error("Bestand is geen geldig IFCX document");
    }
    let envelope: unknown = null;
    for (const entry of parsed.data) {
      const env = entry.attributes?.["isso51::envelope::v1"];
      if (env) {
        envelope = env;
        break;
      }
    }
    if (!envelope) {
      throw new Error("Geen isso51::envelope::v1 in document");
    }
    return {
      content: [{ type: "text", text: JSON.stringify(envelope, null, 2) }],
    };
  },
);

// ----- Tool: get_schema -----------------------------------------------------

server.tool(
  "get_schema",
  "Haal de JSON schema van project of result op (uit schemas/v1/).",
  {
    name: z.enum(["project", "result"]).describe("Welk schema"),
  },
  async ({ name }) => {
    const schemaPath = join(REPO_ROOT, "schemas", "v1", `${name}.schema.json`);
    const content = readFileSync(schemaPath, "utf8");
    return {
      content: [{ type: "text", text: content }],
    };
  },
);

// ----- Tool: list_constructions ---------------------------------------------

server.tool(
  "list_constructions",
  "Lijst van standaard constructies uit de bibliotheek (Rc-waarden, lambdas). Geeft de catalogue.json content terug.",
  {},
  async () => {
    // Catalog leeft in frontend/src/lib/constructionCatalogue als typescript;
    // voor de MCP geven we de geserialiseerde versie uit de schemas folder
    // als die bestaat, anders een placeholder.
    const candidates = [
      join(REPO_ROOT, "schemas", "v1", "construction-catalogue.json"),
      join(REPO_ROOT, "frontend", "src", "lib", "constructionCatalogue.ts"),
    ];
    for (const p of candidates) {
      if (existsSync(p)) {
        const content = readFileSync(p, "utf8");
        return {
          content: [{ type: "text", text: `Source: ${p}\n\n${content}` }],
        };
      }
    }
    return {
      content: [
        {
          type: "text",
          text:
            "Standaard constructies-catalogus niet gevonden in schemas/v1/.\n" +
            "Bekijk frontend/src/lib/constructionCatalogue.ts voor de TypeScript-bron.",
        },
      ],
    };
  },
);

// ----- Resources ------------------------------------------------------------

server.resource(
  "current-project",
  "project://current",
  {
    name: "Current project",
    description: "Het project zoals laatst ingeladen of berekend in deze sessie.",
    mimeType: "application/json",
  },
  async () => ({
    contents: [
      {
        uri: "project://current",
        mimeType: "application/json",
        text: state.project
          ? JSON.stringify(state.project, null, 2)
          : "{ \"info\": null, \"hint\": \"Gebruik calculate_file of calculate om eerst een project te laden.\" }",
      },
    ],
  }),
);

server.resource(
  "current-result",
  "result://current",
  {
    name: "Current result",
    description: "Berekend resultaat van het laatst geladen project.",
    mimeType: "application/json",
  },
  async () => ({
    contents: [
      {
        uri: "result://current",
        mimeType: "application/json",
        text: state.result
          ? JSON.stringify(state.result, null, 2)
          : "{ \"hint\": \"Gebruik calculate of calculate_file om een resultaat te genereren.\" }",
      },
    ],
  }),
);

// ---------------------------------------------------------------------------
// Bootstrap
// ---------------------------------------------------------------------------

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  // stdio mode: log naar stderr zodat stdout schoon blijft voor MCP
  console.error("[ohs-mcp] Open Heatloss Studio MCP server gestart (stdio)");
}

main().catch((err) => {
  console.error("[ohs-mcp] Fatale fout:", err);
  process.exit(1);
});
