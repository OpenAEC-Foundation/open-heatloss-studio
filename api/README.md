# Open Heatloss Studio — REST API

De HTTP-API zit in [`crates/isso51-api`](../crates/isso51-api) (Rust + axum).
Deze map bevat de publieke API-documentatie en client-voorbeelden.

## Endpoints

Base path: `/api/v1`. Productie: `https://warmteverlies.open-aec.com/api/v1`
(forward-auth via Authentik). Lokaal: `http://localhost:3001/api/v1`
(via `cargo run -p isso51-api`).

### Public (no auth)

| Method | Path | Beschrijving |
|---|---|---|
| `GET` | `/health` | Server status + version |
| `POST` | `/calculate` | Bereken warmteverlies uit Project JSON |
| `POST` | `/calculate/ifcx` | Bereken via IFCX (IFC5 alpha) document |
| `GET` | `/schemas/project` | JSON-schema voor Project |
| `GET` | `/schemas/result` | JSON-schema voor ProjectResult |
| `GET` | `/schemas/ifcx` | JSON-schema voor IFCX document |
| `POST` | `/ifc/import` | Upload `.ifc` → server-side import via Python sidecar |
| `POST` | `/import/thermal` | Thermal import (Revit/IFC export JSON) → Project |

### Authenticated (Authentik forward-auth cookie)

| Method | Path | Beschrijving |
|---|---|---|
| `GET` | `/me` | Huidige gebruiker (uit `X-Authentik-*` headers) |
| `GET` | `/projects` | Lijst projecten van user |
| `POST` | `/projects` | Nieuw project aanmaken |
| `GET` | `/projects/{id}` | Project laden |
| `PUT` | `/projects/{id}` | Project bijwerken (optimistic lock via `expected_updated_at`) |
| `DELETE` | `/projects/{id}` | Project soft-delete |
| `POST` | `/projects/{id}/calculate` | Server-side berekenen + opslaan |
| `GET` | `/cloud/status` | Nextcloud cloud-storage beschikbaarheid |
| `GET` | `/cloud/projects` | Projecten uit Nextcloud |
| `GET` | `/cloud/projects/{project}/models` | IFC bestanden in Nextcloud |
| `GET` | `/cloud/projects/{project}/calculations` | Berekeningen in Nextcloud |
| `POST` | `/cloud/projects/{project}/save` | Berekening opslaan + manifest update |
| `POST` | `/report` | PDF rapport genereren via remote service |

## Voorbeelden

### Berekenen via curl (public endpoint)

```bash
curl -X POST http://localhost:3001/api/v1/calculate \
  -H 'Content-Type: application/json' \
  -d @project.json
```

### Schema ophalen

```bash
curl http://localhost:3001/api/v1/schemas/project | jq .
```

### IFCX-route

```bash
curl -X POST http://localhost:3001/api/v1/calculate/ifcx \
  -H 'Content-Type: application/json' \
  -d @ifcenergy-document.json
```

## Lokaal draaien

```bash
# Vanuit repo-root
cargo run --release -p isso51-api
# Server draait op http://localhost:3001
```

Of via docker-compose:

```bash
docker compose up -d
# warmteverlies container exposeert :3001 intern (achter Caddy in productie)
```

## Auth

In productie staat de API achter Caddy met Authentik forward-auth:
- `GET /api/v1/calculate` (en andere `/calculate*`, `/schemas/*`, `/health`)
  zijn publiek (geen auth)
- Alle andere routes vereisen een geldige `authentik_session` cookie
- Het backend leest user-info uit `X-Authentik-Username`, `X-Authentik-Email`,
  `X-Authentik-Uid` headers (die Caddy injecteert na succesvolle handshake)

Lokaal/dev: alle endpoints toegankelijk zonder auth.

## Foutmeldingen

Errors retourneren JSON met `error` + `detail`:

```json
{
  "error": "validation_error",
  "detail": "Verplicht veld 'building' ontbreekt"
}
```

HTTP statuscodes:
- `200` OK
- `400` Bad Request (validatie)
- `401` Unauthorized
- `404` Not Found
- `409` Conflict (project optimistic-lock)
- `500` Internal Server Error (calc fout / panic)

## Bron

API-implementatie: [`crates/isso51-api/src`](../crates/isso51-api/src)

Belangrijkste bestanden:
- [`main.rs`](../crates/isso51-api/src/main.rs) — server bootstrap, route-registratie
- [`handlers/calculation.rs`](../crates/isso51-api/src/handlers/calculation.rs) — `/calculate`, `/health`, `/schemas`
- [`handlers/projects.rs`](../crates/isso51-api/src/handlers/projects.rs) — projecten CRUD
- [`handlers/cloud.rs`](../crates/isso51-api/src/handlers/cloud.rs) — Nextcloud cloud-routes
- [`handlers/ifc_import.rs`](../crates/isso51-api/src/handlers/ifc_import.rs) — `.ifc` upload
- [`handlers/report.rs`](../crates/isso51-api/src/handlers/report.rs) — PDF-proxy naar openaec-reports
- [`auth.rs`](../crates/isso51-api/src/auth.rs) — Authentik forward-auth middleware

## Zie ook

- [MCP server](../mcp-server/) — wrapper rond de calc-engine voor AI-clients
- [`crates/isso51-core`](../crates/isso51-core) — pure rekenkern (zonder HTTP)
- [`crates/isso51-ifcx`](../crates/isso51-ifcx) — IFCX adapter met `isso51::` namespace
