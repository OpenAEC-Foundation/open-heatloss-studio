# Forward-auth trust boundary — verificatie 2026-07-02

## Model

`crates/isso51-api/src/auth.rs` implementeert géén eigen sessie- of tokenvalidatie voor
browser-verkeer. Het vertrouwt volledig op `X-Authentik-*` headers die de reverse proxy
meelevert nadat Authentik het request heeft geauthenticeerd (forward-auth pattern). Dit
is alleen veilig zolang de proxy (Caddy) garandeert dat:

1. elk request eerst door de Authentik-outpost wordt geautoriseerd, en
2. eventuele `X-Authentik-*`-headers die de *client zelf* meestuurt, nooit ongewijzigd
   doorgezet worden naar de backend — anders kan een client zijn eigen identiteit/rol
   spoofen (bijv. `X-Authentik-Groups: admins` of `X-Authentik-Meta-Tenant: andere-klant`).

De backend zelf heeft geen zicht op de Caddy-config; deze garantie leeft uitsluitend in
`/opt/openaec/caddy/Caddyfile` op de server.

## Caddy-garantie — relevante config

```caddyfile
(authentik_forward_auth) {
	reverse_proxy /outpost.goauthentik.io/* http://authentik-server:9000

	@needs_auth {
		not path /api/health
		not header Authorization Bearer*
		not header X-API-Key *
		not method OPTIONS
	}
	forward_auth @needs_auth http://authentik-server:9000 {
		uri /outpost.goauthentik.io/auth/caddy
		copy_headers X-Authentik-Username X-Authentik-Groups X-Authentik-Email X-Authentik-Name X-Authentik-Uid X-Authentik-Meta-Username X-Authentik-Meta-Email X-Authentik-Meta-Name X-Authentik-Meta-Groups X-Authentik-Meta-Tenant X-Authentik-Meta-Company X-Authentik-Meta-JobTitle X-Authentik-Meta-Phone X-Authentik-Meta-RegNumber
		trusted_proxies private_ranges
	}
}

warmteverlies.open-aec.com {
	route {
		import authentik_forward_auth
		@all path /*
		reverse_proxy @all warmteverlies:3001
	}
	...
}
```

`warmteverlies:3001` (de `isso51-api`-container) is **niet** extern gepubliceerd — de
compose-service exposet alleen intern (`expose: 3001`, geen `ports:`), dus het is
onbereikbaar behalve via het Docker-netwerk vanaf Caddy. Dit is een tweede
verdedigingslaag naast de header-strip hieronder.

## Bevinding: Caddy's `copy_headers` had zelf een spoofing-CVE (GHSA-7r4p-vjf4-gxv4)

De config-directive `copy_headers` is *bedoeld* om precies dit te garanderen: elke header
in de lijst wordt eerst uit het inkomende (client-)request verwijderd, en pas daarna
(conditioneel) gezet met de waarde uit de Authentik-response. Echter: deze
delete-before-set logica ontbrak in Caddy vóór commit `2dbcdefb` (PR #7545, gemerged
2026-03-04, gefixt als GHSA-7r4p-vjf4-gxv4). Zonder de fix geldt: als de Authentik-outpost
een specifieke header (bijv. `X-Authentik-Meta-RegNumber` of `X-Authentik-Meta-Tenant`)
niet meestuurt in zijn auth-response — bijvoorbeeld omdat het betreffende profielveld bij
een gebruiker leeg is — dan slaat Caddy de `Set` over, maar verwijdert het de
client-aangeleverde waarde ook niet. Een ingelogde gebruiker met een leeg profielveld kon
zo zijn eigen `X-Authentik-Meta-Tenant`- of `X-Authentik-Groups`-header spoofen, en die
waarde bereikte ongewijzigd de backend.

`crates/isso51-api/src/auth.rs` gebruikt zowel `HEADER_GROUPS` (autorisatie/rol-checks)
als `HEADER_TENANT` (tenant-isolatie in `apply_tenant_override`) — dit is dus geen
theoretisch risico maar direct bruikbaar voor tenant-crossover of privilege-escalatie
door een reeds ingelogde gebruiker.

**Status vóór deze sessie:** productie draaide Caddy `v2.10.2` (uitgebracht 2025-08-23,
dus **vóór** de fix van 2026-03-04) — kwetsbaar.

**Actie in deze sessie (2026-07-02):**
```bash
sudo docker compose pull caddy
sudo docker compose up -d caddy
```
Resultaat: Caddy geüpgraded naar `v2.11.4` (bevestigd 94 commits ná de fix-commit via
`git compare`, dus de patch is aanwezig). Containers `caddy` en `warmteverlies` beide
`healthy` na herstart, geen errors in `docker logs caddy`. Caddyfile zelf is
**niet gewijzigd** — de bestaande `copy_headers`-config was al correct, alleen de
binary-versie was het probleem.

## Empirische verificatie (2026-07-02, ná de Caddy-upgrade)

```
curl -s -o /dev/null -w "%{http_code}" https://warmteverlies.open-aec.com/api/v1/me
→ 302 (redirect naar auth.open-aec.com/application/o/authorize/...)

curl -s -o /dev/null -w "%{http_code}" https://warmteverlies.open-aec.com/api/v1/me \
  -H "X-Authentik-Username: spoofadmin"
→ 302 (zelfde redirect, géén 200 — geen bypass)

curl -s -o /dev/null -w "%{http_code}" https://warmteverlies.open-aec.com/api/v1/me \
  -H "X-Authentik-Groups: admins"
→ 302 (zelfde redirect)
```

**Beperking van deze test:** zonder geldige Authentik-sessie (cookie) reageert de outpost
altijd met een 302-redirect vóórdat de request de `copy_headers`-logica bereikt — dit
bewijst dat er geen ongeauthenticeerde bypass is, maar toont *niet* het specifieke
CVE-scenario (een reeds ingelogde gebruiker met een leeg profielveld die een náást
gelegen header spoofed). Dat scenario is nu wel afgedekt doordat de patch aantoonbaar in
de draaiende binary zit (bronverificatie via Caddy's `forwardauth/caddyfile.go`, regel
216-221: "Always delete the client-supplied header before conditionally setting it").

Overige vhosts (`report.open-aec.com`, `bcf.open-aec.com`, `auth.open-aec.com`) na de
Caddy-herstart getest — allemaal 302 naar login zoals verwacht, geen regressie.

## Antwoorden op de audit-vragen

| Vraag | Antwoord |
|---|---|
| (a) Forward-auth actief op warmteverlies-vhost? | Ja — `route { import authentik_forward_auth ... }` |
| (b) Worden client-`X-Authentik-*`-headers gestript vóór doorzetten? | Ja, via `copy_headers`. Was **niet effectief** op de draaiende Caddy-versie (v2.10.2, CVE GHSA-7r4p-vjf4-gxv4) — gefixt door upgrade naar v2.11.4. |
| (c) Empirische spoof-test | 302 in alle gevallen (geen sessie beschikbaar); geen 200-bypass waargenomen. Volledige CVE-scenario alleen dicht te bewijzen via broncode-verificatie van de patch, zie hierboven. |

## Open item — defense-in-depth aanbeveling

De huidige garantie steunt volledig op (1) Caddy-versie correct gepatcht blijven en
(2) `warmteverlies:3001` nooit extern gepubliceerd worden. Beide zijn config-afhankelijk
en niet zelf-verifiërend vanuit de backend. Aanbeveling: introduceer een gedeeld secret
(bijv. `X-Internal-Proxy-Secret`, willekeurige waarde in `.env`, gezet door Caddy via
`header_up` en gevalideerd in `auth.rs` vóór de `X-Authentik-*`-headers vertrouwd worden).
Dit maakt de trust-boundary expliciet controleerbaar in de backend-code zelf, onafhankelijk
van Caddy-versie of netwerktopologie-aannames. Nog niet geïmplementeerd — buiten scope van
deze verificatie-sessie.
