//! Authentik forward_auth based authentication.
//!
//! Trust model: Caddy + Authentik proxy outpost authenticate the request
//! _before_ it reaches this service and inject `X-Authentik-*` headers with
//! the user's identity and tenant claims. The upstream containers are only
//! reachable via the internal Docker network, so we trust these headers.
//!
//! See `docs/2026-04-16-authentik-unified-sso-plan.md` §2.2 / §5.1 for the
//! migration rationale (replaces the previous OIDC JWKS validator).
//!
//! Bearer-token validation for machine clients (`Authorization: Bearer ak_*`
//! against Authentik's token endpoint with a small in-memory cache) is
//! planned for fase 6 of the SSO migration plan and currently returns 401.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

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

// ---------------------------------------------------------------------------
// Claims
// ---------------------------------------------------------------------------

/// Authenticated user claims, populated from the `X-Authentik-*` request
/// headers that Caddy injects after successful forward_auth.
///
/// Field names mirror the previous OIDC implementation so existing handlers
/// (`projects.rs`, `user.rs`, `cloud.rs`, …) keep compiling unchanged.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OidcClaims {
    /// Subject identifier — primary key for the user record.
    /// Sourced from `X-Authentik-Username` (stable, human-readable).
    pub sub: String,
    /// User's email address (`X-Authentik-Email`).
    pub email: Option<String>,
    /// User's display name (`X-Authentik-Name`).
    pub name: Option<String>,
    /// Preferred username — same as `sub` for Authentik forward_auth.
    pub preferred_username: Option<String>,
    /// Issuer label kept for backward compatibility with the `users` table
    /// (column `oidc_issuer`). Constant value `"authentik"` after migration.
    pub iss: Option<String>,
    /// Group memberships (parsed from `X-Authentik-Groups`).
    pub groups: Vec<String>,
    /// Tenant slug from `X-Authentik-Meta-Tenant`.
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
    /// Bearer token flow not yet implemented (fase 6 of the SSO migration).
    BearerNotImplemented,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::MissingHeaders => (
                StatusCode::UNAUTHORIZED,
                "Authenticatie ontbreekt — login via Authentik vereist",
            ),
            AuthError::BearerNotImplemented => (
                StatusCode::UNAUTHORIZED,
                "Bearer-token authenticatie nog niet geactiveerd voor deze service",
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
// Header parsing
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
            raw.split(|c: char| c == ',' || c == '|' || c == ';')
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
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_start().starts_with("Bearer ak-"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Axum extractor
// ---------------------------------------------------------------------------

/// Axum extractor that pulls user identity from the `X-Authentik-*` headers
/// injected by Caddy after a successful forward_auth handshake.
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
        // Bearer token path — fase 6 of the SSO migration. Bypass forward_auth
        // happens in Caddy; the upstream is then responsible for validating
        // the token against Authentik. Until that lands we explicitly reject
        // so callers don't silently fall through into the header path.
        if is_bearer_token(&parts.headers) {
            // TODO(fase 6): validate token against Authentik
            // `/api/v3/core/tokens/<identifier>/view_key/`, cache 5 min TTL.
            // See SSO plan §2.3 + §5.1.
            return Err(AuthError::BearerNotImplemented);
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

    #[test]
    fn detects_bearer_ak_prefix() {
        let headers = hdr(&[("authorization", "Bearer ak-test-token")]);
        assert!(is_bearer_token(&headers));

        let headers = hdr(&[("authorization", "Bearer some-jwt")]);
        assert!(!is_bearer_token(&headers));
    }
}
