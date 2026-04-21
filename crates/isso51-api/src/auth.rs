//! Authentik forward_auth + Bearer token based authentication.
//!
//! Trust model for browser traffic: Caddy + Authentik proxy outpost
//! authenticate the request _before_ it reaches this service and inject
//! `X-Authentik-*` headers with the user's identity and tenant claims.
//! The upstream containers are only reachable via the internal Docker
//! network, so we trust these headers.
//!
//! Machine-clients (PyRevit exporter, CI, externe consumers) sturen een
//! `Authorization: Bearer ak-*` header. De Caddy forward_auth laat deze
//! requests bypass maken zodat de header onaangeroerd bij deze service
//! binnenkomt. Validatie gebeurt tegen Authentik's `/api/v3/core/users/me/`
//! endpoint met een 5-minuten fingerprint-cache — zie
//! [`validate_authentik_token`].
//!
//! Optionele header `X-Original-Tenant` wordt gehonoreerd voor
//! backend-to-backend impersonation: Authentik heeft de caller al
//! gevalideerd, dus de service mag op naam van een andere tenant spreken.
//!
//! Referenties:
//! - `docs/2026-04-16-authentik-unified-sso-plan.md` §2.2 / §5.1
//! - `openaec-reports` Python referentie: `auth/dependencies.py`
//!   (`_cached_authentik_user`, `_authenticate_via_authentik_token`)
//! - BCF peer-implementatie:
//!   `crates/bcf-server/src/auth/authentik_token.rs`

use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Header names
// ---------------------------------------------------------------------------

/// Stable user identifier (Authentik username, used as primary key).
pub const HEADER_USERNAME: &str = "X-Authentik-Username";
/// Email address.
pub const HEADER_EMAIL: &str = "X-Authentik-Email";
/// Display name.
pub const HEADER_NAME: &str = "X-Authentik-Name";
/// Authentik UID (sub claim equivalent — UUID string).
pub const HEADER_UID: &str = "X-Authentik-Uid";
/// Comma-separated group list.
pub const HEADER_GROUPS: &str = "X-Authentik-Groups";
/// Tenant slug (custom property mapping `openaec-tenant-headers`).
pub const HEADER_TENANT: &str = "X-Authentik-Meta-Tenant";
/// Company name (custom).
pub const HEADER_COMPANY: &str = "X-Authentik-Meta-Company";
/// Job title (custom).
pub const HEADER_JOB_TITLE: &str = "X-Authentik-Meta-JobTitle";
/// Phone number (custom).
pub const HEADER_PHONE: &str = "X-Authentik-Meta-Phone";
/// Registration number (custom).
pub const HEADER_REG_NUMBER: &str = "X-Authentik-Meta-RegNumber";

/// Impersonation header for backend-to-backend calls — overrides tenant
/// claim when the caller has authenticated via a Bearer service token.
pub const HEADER_ORIGINAL_TENANT: &str = "X-Original-Tenant";

// ---------------------------------------------------------------------------
// Claims
// ---------------------------------------------------------------------------

/// Authenticated user claims, populated from either the `X-Authentik-*`
/// forward_auth headers or an Authentik-validated service-token.
///
/// Field names mirror the previous OIDC implementation so existing handlers
/// (`projects.rs`, `user.rs`, `cloud.rs`, …) keep compiling unchanged.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OidcClaims {
    /// Subject identifier — primary key for the user record.
    /// Sourced from `X-Authentik-Username` (stable, human-readable) or
    /// from the Authentik `username` field when validating via Bearer.
    pub sub: String,
    /// User's email address (`X-Authentik-Email` or Authentik `email`).
    pub email: Option<String>,
    /// User's display name (`X-Authentik-Name` or Authentik `name`).
    pub name: Option<String>,
    /// Preferred username — same as `sub` for Authentik.
    pub preferred_username: Option<String>,
    /// Issuer label kept for backward compatibility with the `users` table
    /// (column `oidc_issuer`). Constant value `"authentik"` after migration.
    pub iss: Option<String>,
    /// Group memberships (parsed from `X-Authentik-Groups`).
    pub groups: Vec<String>,
    /// Tenant slug from `X-Authentik-Meta-Tenant` / `attributes.tenant`,
    /// optioneel overschreven door `X-Original-Tenant`.
    pub tenant: Option<String>,
    /// Company from `X-Authentik-Meta-Company`.
    pub company: Option<String>,
    /// Job title from `X-Authentik-Meta-JobTitle`.
    pub job_title: Option<String>,
    /// Phone from `X-Authentik-Meta-Phone`.
    pub phone: Option<String>,
    /// Registration number from `X-Authentik-Meta-RegNumber`.
    pub registration_number: Option<String>,
    /// Authentik UID (UUID). Optional — `sub` is used as PK.
    pub uid: Option<String>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Authentication error returned to the client.
#[derive(Debug)]
pub enum AuthError {
    /// Request had no `X-Authentik-Username` header — Caddy/Authentik did
    /// not authenticate the caller. Should never happen for browser traffic
    /// behind the configured forward_auth.
    MissingHeaders,
    /// `Authorization: Bearer ak-*` token was supplied, but Authentik
    /// rejected it (expired, revoked, unknown) or the validation call
    /// failed (timeout, network).
    InvalidBearerToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::MissingHeaders => (
                StatusCode::UNAUTHORIZED,
                "Authenticatie ontbreekt — login via Authentik vereist",
            ),
            AuthError::InvalidBearerToken => (
                StatusCode::UNAUTHORIZED,
                "Bearer-token ongeldig — controleer of het Authentik service-token actief is",
            ),
        };

        (
            status,
            Json(serde_json::json!({
                "error": "unauthorized",
                "detail": msg,
            })),
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// Header parsing (forward_auth flow)
// ---------------------------------------------------------------------------

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    header_str(headers, name).map(str::to_string)
}

fn parse_groups(headers: &HeaderMap) -> Vec<String> {
    header_str(headers, HEADER_GROUPS)
        .map(|raw| {
            // Authentik separates groups by `|` historically, comma-list is
            // also seen depending on the forward_auth path. Accept both.
            raw.split([',', '|', ';'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Build `OidcClaims` from request headers. Returns `Err` when the required
/// `X-Authentik-Username` header is missing.
fn claims_from_headers(headers: &HeaderMap) -> Result<OidcClaims, AuthError> {
    let username = header_string(headers, HEADER_USERNAME).ok_or(AuthError::MissingHeaders)?;

    Ok(OidcClaims {
        sub: username.clone(),
        email: header_string(headers, HEADER_EMAIL),
        name: header_string(headers, HEADER_NAME),
        preferred_username: Some(username),
        iss: Some("authentik".to_string()),
        groups: parse_groups(headers),
        tenant: header_string(headers, HEADER_TENANT),
        company: header_string(headers, HEADER_COMPANY),
        job_title: header_string(headers, HEADER_JOB_TITLE),
        phone: header_string(headers, HEADER_PHONE),
        registration_number: header_string(headers, HEADER_REG_NUMBER),
        uid: header_string(headers, HEADER_UID),
    })
}

/// Detects an Authentik service-account bearer token.
///
/// Format: `Authorization: Bearer ak-<token>` (the `ak-` prefix is the
/// Authentik convention for application-keys).
fn is_bearer_token(headers: &HeaderMap) -> bool {
    extract_bearer_token(headers)
        .map(|t| t.starts_with("ak-"))
        .unwrap_or(false)
}

/// Extract the raw token value from an `Authorization: Bearer ...` header.
fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Authentik Bearer token validation
// ---------------------------------------------------------------------------

/// How long a positive Authentik `users/me` response stays cached.
const TOKEN_CACHE_TTL: Duration = Duration::from_secs(300);

/// HTTP timeout for the Authentik validation round-trip.
const AUTHENTIK_TIMEOUT: Duration = Duration::from_secs(5);

/// Default Authentik instance — overridable via `AUTHENTIK_API_URL`.
const DEFAULT_AUTHENTIK_URL: &str = "https://auth.open-aec.com";

/// Cached user-info payload extracted from `/api/v3/core/users/me/`.
#[derive(Debug, Clone)]
struct AuthentikUserInfo {
    /// Authentik `username` field — stable identifier, maps to `sub`.
    username: String,
    /// Authentik `name` field — display name.
    display_name: Option<String>,
    /// Authentik `email` field.
    email: Option<String>,
    /// Authentik user `pk` — UUID that maps to `OidcClaims::uid`.
    pk: Option<String>,
    /// Tenant slug from `attributes.tenant`, if any.
    tenant: Option<String>,
}

/// Single cache entry: instant of insertion + (possibly negative) result.
#[derive(Clone)]
struct CacheEntry {
    inserted: Instant,
    user_info: Option<AuthentikUserInfo>,
}

/// Process-wide cache of token-fingerprint → Authentik user payload.
///
/// `tokio::sync::RwLock` because moka is not in the workspace and the
/// cache sees modest traffic (one lookup per machine-client request).
/// Keyed on the first 16 hex chars of the sha256 fingerprint so we never
/// hold raw tokens in memory.
static TOKEN_CACHE: OnceLock<RwLock<HashMap<String, CacheEntry>>> = OnceLock::new();

/// Shared `reqwest` client re-used across validation calls.
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Cached Authentik base URL, loaded once from env on first use.
static AUTHENTIK_API_URL: OnceLock<String> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<String, CacheEntry>> {
    TOKEN_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(AUTHENTIK_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

fn authentik_api_url() -> &'static str {
    AUTHENTIK_API_URL.get_or_init(|| {
        env::var("AUTHENTIK_API_URL")
            .ok()
            .map(|v| v.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_AUTHENTIK_URL.to_string())
    })
}

/// Return the 16-char sha256 fingerprint of a token.
///
/// Hex-lowercase, first 16 characters — 64 bits of fingerprint, enough to
/// avoid accidental collisions in the process-local cache without exposing
/// the raw token in log lines.
pub fn token_fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let mut out = String::with_capacity(16);
    for byte in &digest[..8] {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Return the cache bucket index for a wall-clock Unix timestamp.
///
/// Buckets are `TOKEN_CACHE_TTL`-wide windows aligned to the Unix epoch —
/// same semantics as the Python reference (`_TOKEN_CACHE_TTL` / 300 s).
/// The production path uses [`Instant::elapsed`] on the cached entry, so
/// this helper exists only for deterministic unit testing of the bucket
/// boundary logic.
#[cfg(test)]
fn current_bucket(now_unix_secs: u64) -> u64 {
    now_unix_secs / TOKEN_CACHE_TTL.as_secs()
}

/// Validate an Authentik `ak-*` Bearer token.
///
/// Returns `Some(OidcClaims)` when Authentik accepts the token, `None`
/// otherwise (rejected by Authentik, HTTP failure, payload mismatch).
/// Both outcomes are cached for [`TOKEN_CACHE_TTL`] on the token's
/// sha256-fingerprint.
pub async fn validate_authentik_token(token: &str) -> Option<OidcClaims> {
    if !token.starts_with("ak-") {
        return None;
    }

    let fingerprint = token_fingerprint(token);

    // 1. Cache fast-path.
    if let Some(cached) = read_cache(&fingerprint).await {
        return cached.map(|info| claims_from_authentik_user(&info));
    }

    // 2. Live validation against Authentik.
    let endpoint = format!("{}/api/v3/core/users/me/", authentik_api_url());

    let response = http_client()
        .get(&endpoint)
        .bearer_auth(token)
        .send()
        .await;

    let user_info = match response {
        Ok(resp) if resp.status().is_success() => parse_users_me_payload(resp).await,
        Ok(resp) => {
            tracing::debug!(status = %resp.status(), fp = %fingerprint, "authentik token rejected");
            None
        }
        Err(err) => {
            tracing::warn!(error = %err, fp = %fingerprint, "authentik token validation error");
            None
        }
    };

    // 3. Cache the outcome (positive or negative).
    write_cache(&fingerprint, user_info.clone()).await;

    user_info.as_ref().map(claims_from_authentik_user)
}

async fn read_cache(fingerprint: &str) -> Option<Option<AuthentikUserInfo>> {
    let guard = cache().read().await;
    let entry = guard.get(fingerprint)?;
    if entry.inserted.elapsed() >= TOKEN_CACHE_TTL {
        return None;
    }
    Some(entry.user_info.clone())
}

async fn write_cache(fingerprint: &str, user_info: Option<AuthentikUserInfo>) {
    let mut guard = cache().write().await;
    // Opportunistic purge of expired entries to bound memory growth.
    guard.retain(|_, entry| entry.inserted.elapsed() < TOKEN_CACHE_TTL);
    guard.insert(
        fingerprint.to_string(),
        CacheEntry {
            inserted: Instant::now(),
            user_info,
        },
    );
}

/// Parse the subset of `/api/v3/core/users/me/` we actually need.
///
/// Authentik returns either `{"user": {...}}` or the bare user object
/// depending on the endpoint variant — tolerate both.
async fn parse_users_me_payload(resp: reqwest::Response) -> Option<AuthentikUserInfo> {
    let body: serde_json::Value = resp.json().await.ok()?;
    let user = body.get("user").unwrap_or(&body);

    let username = user
        .get("username")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;

    let display_name = user
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let pk = user
        .get("pk")
        .map(|v| match v {
            serde_json::Value::String(s) => s.trim().to_string(),
            other => other.to_string(),
        })
        .filter(|s| !s.is_empty());

    let tenant = user
        .get("attributes")
        .and_then(|v| v.get("tenant"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Some(AuthentikUserInfo {
        username,
        display_name,
        email,
        pk,
        tenant,
    })
}

/// Project an [`AuthentikUserInfo`] onto [`OidcClaims`].
///
/// The `tenant` is taken straight from Authentik; the caller (extractor)
/// may override it using the `X-Original-Tenant` request header.
fn claims_from_authentik_user(info: &AuthentikUserInfo) -> OidcClaims {
    OidcClaims {
        sub: info.username.clone(),
        email: info.email.clone(),
        name: info.display_name.clone(),
        preferred_username: Some(info.username.clone()),
        iss: Some("authentik".to_string()),
        groups: Vec::new(),
        tenant: info.tenant.clone(),
        company: None,
        job_title: None,
        phone: None,
        registration_number: None,
        uid: info.pk.clone(),
    }
}

// ---------------------------------------------------------------------------
// Axum extractor
// ---------------------------------------------------------------------------

/// Axum extractor that resolves user identity from either the
/// `X-Authentik-*` forward_auth headers or an `Authorization: Bearer ak-*`
/// service token.
///
/// Usage in handlers (unchanged from the previous JWT implementation):
/// ```ignore
/// async fn my_handler(AuthClaims(claims): AuthClaims) -> impl IntoResponse { ... }
/// ```
pub struct AuthClaims(pub OidcClaims);

impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Bearer token path — machine-client flow (PyRevit, CI, externe
        // consumers). Caddy laat deze requests bypass maken zodat de
        // Authorization header onaangeroerd bij ons binnenkomt.
        if is_bearer_token(&parts.headers) {
            let token = extract_bearer_token(&parts.headers)
                .ok_or(AuthError::InvalidBearerToken)?;
            let mut claims = validate_authentik_token(token)
                .await
                .ok_or(AuthError::InvalidBearerToken)?;

            // X-Original-Tenant is een expliciete impersonation-hint voor
            // backend-to-backend calls. Alleen gehonoreerd nadat Authentik
            // het Bearer-token heeft geaccepteerd.
            if let Some(override_tenant) =
                header_string(&parts.headers, HEADER_ORIGINAL_TENANT)
            {
                claims.tenant = Some(override_tenant);
            }

            return Ok(AuthClaims(claims));
        }

        let claims = claims_from_headers(&parts.headers)?;
        Ok(AuthClaims(claims))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};

    fn hdr(values: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in values {
            h.insert(
                HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        h
    }

    // --- forward_auth header flow -------------------------------------------

    #[test]
    fn missing_username_header_rejects() {
        let headers = hdr(&[(HEADER_EMAIL, "x@y")]);
        assert!(matches!(
            claims_from_headers(&headers),
            Err(AuthError::MissingHeaders)
        ));
    }

    #[test]
    fn populates_all_fields() {
        let headers = hdr(&[
            (HEADER_USERNAME, "jochem"),
            (HEADER_EMAIL, "j@open-aec.com"),
            (HEADER_NAME, "Jochem K"),
            (HEADER_UID, "uuid-123"),
            (HEADER_GROUPS, "admins,builders"),
            (HEADER_TENANT, "3bm"),
            (HEADER_COMPANY, "3BM"),
            (HEADER_JOB_TITLE, "Engineer"),
            (HEADER_PHONE, "+31"),
            (HEADER_REG_NUMBER, "REG-1"),
        ]);

        let claims = claims_from_headers(&headers).expect("claims");
        assert_eq!(claims.sub, "jochem");
        assert_eq!(claims.email.as_deref(), Some("j@open-aec.com"));
        assert_eq!(claims.name.as_deref(), Some("Jochem K"));
        assert_eq!(claims.uid.as_deref(), Some("uuid-123"));
        assert_eq!(claims.groups, vec!["admins", "builders"]);
        assert_eq!(claims.tenant.as_deref(), Some("3bm"));
        assert_eq!(claims.iss.as_deref(), Some("authentik"));
        assert_eq!(claims.preferred_username.as_deref(), Some("jochem"));
    }

    #[test]
    fn groups_accepts_pipe_separator() {
        let headers = hdr(&[
            (HEADER_USERNAME, "u"),
            (HEADER_GROUPS, "alpha|beta|gamma"),
        ]);
        let claims = claims_from_headers(&headers).expect("claims");
        assert_eq!(claims.groups, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn empty_header_values_treated_as_missing() {
        let headers = hdr(&[(HEADER_USERNAME, "  "), (HEADER_EMAIL, "")]);
        assert!(matches!(
            claims_from_headers(&headers),
            Err(AuthError::MissingHeaders)
        ));
    }

    // --- Bearer prefix detection --------------------------------------------

    #[test]
    fn detects_bearer_ak_prefix() {
        let headers = hdr(&[("authorization", "Bearer ak-test-token")]);
        assert!(is_bearer_token(&headers));

        let headers = hdr(&[("authorization", "Bearer some-jwt")]);
        assert!(!is_bearer_token(&headers));

        // Basic auth must not trigger the bearer path.
        let headers = hdr(&[("authorization", "Basic ak-bogus")]);
        assert!(!is_bearer_token(&headers));

        // Missing header.
        let headers = hdr(&[]);
        assert!(!is_bearer_token(&headers));
    }

    #[test]
    fn extract_bearer_strips_prefix_and_trims() {
        let headers = hdr(&[("authorization", "Bearer ak-foo-bar  ")]);
        assert_eq!(extract_bearer_token(&headers), Some("ak-foo-bar"));
    }

    // --- Fingerprint --------------------------------------------------------

    #[test]
    fn fingerprint_is_16_hex_chars_and_stable() {
        let fp = token_fingerprint("ak-example-token");
        assert_eq!(fp.len(), 16);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
        // Deterministic — same input, same output.
        assert_eq!(fp, token_fingerprint("ak-example-token"));
        // Different inputs yield different fingerprints.
        assert_ne!(fp, token_fingerprint("ak-other-token"));
    }

    // --- Cache TTL bucket ---------------------------------------------------

    #[test]
    fn bucket_changes_every_ttl_window() {
        let ttl = TOKEN_CACHE_TTL.as_secs();
        // Anchor to an exact bucket boundary so `base..base+ttl-1` spans a single bucket.
        let base: u64 = (1_700_000_000 / ttl) * ttl;
        assert_eq!(current_bucket(base), current_bucket(base + ttl - 1));
        assert_ne!(current_bucket(base), current_bucket(base + ttl));
        // Adjacent buckets differ by exactly one.
        assert_eq!(current_bucket(base + ttl), current_bucket(base) + 1);
    }

    #[tokio::test]
    async fn cache_returns_none_after_ttl_expiry() {
        let fp = "cafebabecafebabe";
        {
            let mut guard = cache().write().await;
            guard.insert(
                fp.to_string(),
                CacheEntry {
                    inserted: Instant::now() - TOKEN_CACHE_TTL - Duration::from_secs(1),
                    user_info: Some(AuthentikUserInfo {
                        username: "expired".into(),
                        display_name: Some("Expired".into()),
                        email: None,
                        pk: None,
                        tenant: None,
                    }),
                },
            );
        }
        assert!(read_cache(fp).await.is_none());
        cache().write().await.remove(fp);
    }

    #[tokio::test]
    async fn cache_roundtrips_positive_and_negative_entries() {
        let positive_fp = "1111111111111111";
        let negative_fp = "2222222222222222";

        write_cache(
            positive_fp,
            Some(AuthentikUserInfo {
                username: "svc-user".into(),
                display_name: Some("Service User".into()),
                email: Some("svc@service.openaec.local".into()),
                pk: Some("abcd-1234".into()),
                tenant: Some("3bm".into()),
            }),
        )
        .await;
        write_cache(negative_fp, None).await;

        let hit = read_cache(positive_fp).await.expect("present");
        let info = hit.expect("positive");
        assert_eq!(info.username, "svc-user");
        assert_eq!(info.tenant.as_deref(), Some("3bm"));
        assert_eq!(info.email.as_deref(), Some("svc@service.openaec.local"));

        let miss = read_cache(negative_fp).await.expect("present");
        assert!(miss.is_none());

        cache().write().await.remove(positive_fp);
        cache().write().await.remove(negative_fp);
    }

    // --- Claims projection + tenant override --------------------------------

    #[test]
    fn claims_from_authentik_user_maps_fields() {
        let info = AuthentikUserInfo {
            username: "svc-warmteverlies".into(),
            display_name: Some("Warmteverlies Service".into()),
            email: Some("svc@service.openaec.local".into()),
            pk: Some("uuid-pk-7".into()),
            tenant: Some("3bm".into()),
        };
        let claims = claims_from_authentik_user(&info);
        assert_eq!(claims.sub, "svc-warmteverlies");
        assert_eq!(claims.preferred_username.as_deref(), Some("svc-warmteverlies"));
        assert_eq!(claims.email.as_deref(), Some("svc@service.openaec.local"));
        assert_eq!(claims.name.as_deref(), Some("Warmteverlies Service"));
        assert_eq!(claims.uid.as_deref(), Some("uuid-pk-7"));
        assert_eq!(claims.tenant.as_deref(), Some("3bm"));
        assert_eq!(claims.iss.as_deref(), Some("authentik"));
    }

    /// Simulates the extractor logic for tenant override — we can't spin
    /// up Authentik in a unit test, so we assert the override step in
    /// isolation: start with claims that have tenant `3bm`, apply the
    /// `X-Original-Tenant` header logic, and verify the override wins.
    #[test]
    fn x_original_tenant_overrides_authentik_claim() {
        let info = AuthentikUserInfo {
            username: "svc".into(),
            display_name: None,
            email: None,
            pk: None,
            tenant: Some("3bm".into()),
        };
        let mut claims = claims_from_authentik_user(&info);

        let headers = hdr(&[(HEADER_ORIGINAL_TENANT, "klant-a")]);
        if let Some(override_tenant) = header_string(&headers, HEADER_ORIGINAL_TENANT) {
            claims.tenant = Some(override_tenant);
        }
        assert_eq!(claims.tenant.as_deref(), Some("klant-a"));

        // Zonder header blijft de Authentik-waarde staan.
        let info2 = AuthentikUserInfo {
            username: "svc".into(),
            display_name: None,
            email: None,
            pk: None,
            tenant: Some("3bm".into()),
        };
        let mut claims2 = claims_from_authentik_user(&info2);
        let headers2 = hdr(&[]);
        if let Some(override_tenant) = header_string(&headers2, HEADER_ORIGINAL_TENANT) {
            claims2.tenant = Some(override_tenant);
        }
        assert_eq!(claims2.tenant.as_deref(), Some("3bm"));
    }

    #[tokio::test]
    async fn validate_authentik_token_rejects_non_ak_prefix() {
        // Tokens zonder `ak-` prefix moeten direct None teruggeven zonder
        // een HTTP call te doen — de Bearer-detectie is al gebeurd, deze
        // extra gate beschermt tegen misbruik van de helper.
        assert!(validate_authentik_token("eyJfake").await.is_none());
        assert!(validate_authentik_token("random-token").await.is_none());
    }
}
