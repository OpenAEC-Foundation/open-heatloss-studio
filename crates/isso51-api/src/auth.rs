//! OIDC JWT authentication: JWKS discovery, token validation, and Axum extractor.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Claims extracted from a validated OIDC JWT token.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OidcClaims {
    /// Subject identifier (unique user ID from the identity provider).
    pub sub: String,
    /// User's email address.
    pub email: Option<String>,
    /// User's display name.
    pub name: Option<String>,
    /// Preferred username.
    pub preferred_username: Option<String>,
    /// Token issuer URL.
    pub iss: Option<String>,
}

/// OIDC configuration discovered from the issuer.
#[derive(Debug, Deserialize)]
struct OidcDiscovery {
    /// Canonical issuer URL (used for JWT `iss` claim validation).
    issuer: String,
    jwks_uri: String,
}

/// JSON Web Key Set.
#[derive(Clone, Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

/// A single JSON Web Key.
#[derive(Clone, Debug, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    n: Option<String>,
    e: Option<String>,
}

/// Shared JWKS cache, refreshed periodically.
#[derive(Clone)]
pub struct JwksCache {
    jwks_uri: String,
    audience: String,
    issuer: String,
    keys: Arc<RwLock<Vec<Jwk>>>,
}

impl JwksCache {
    /// Create a new JWKS cache by discovering the OIDC configuration.
    pub async fn from_issuer(issuer: &str, audience: &str) -> Result<Self, String> {
        // Normalize issuer URL (remove trailing slash for discovery endpoint).
        let issuer_trimmed = issuer.trim_end_matches('/');
        let discovery_url =
            format!("{issuer_trimmed}/.well-known/openid-configuration");

        tracing::info!("Fetching OIDC discovery from {discovery_url}");

        let discovery: OidcDiscovery = reqwest::get(&discovery_url)
            .await
            .map_err(|e| format!("Failed to fetch OIDC discovery: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse OIDC discovery: {e}"))?;

        // Use the canonical issuer from the discovery document for JWT validation.
        // This ensures the `iss` claim in tokens matches exactly (trailing slash etc.).
        tracing::info!(
            "OIDC issuer (canonical): {}, JWKS URI: {}",
            discovery.issuer,
            discovery.jwks_uri
        );

        let jwks: Jwks = reqwest::get(&discovery.jwks_uri)
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {e}"))?;

        tracing::info!("Loaded {} JWKS keys", jwks.keys.len());

        Ok(Self {
            jwks_uri: discovery.jwks_uri,
            audience: audience.to_string(),
            issuer: discovery.issuer,
            keys: Arc::new(RwLock::new(jwks.keys)),
        })
    }

    /// Validate a JWT token and extract claims.
    pub async fn validate_token(&self, token: &str) -> Result<OidcClaims, AuthError> {
        let header =
            decode_header(token).map_err(|_| AuthError::InvalidToken)?;

        let kid = header.kid.as_deref();

        // Find matching key.
        let keys = self.keys.read().await;
        let jwk = find_key(&keys, kid).ok_or(AuthError::KeyNotFound)?;

        let decoding_key = DecodingKey::from_rsa_components(
            jwk.n.as_deref().ok_or(AuthError::InvalidToken)?,
            jwk.e.as_deref().ok_or(AuthError::InvalidToken)?,
        )
        .map_err(|_| AuthError::InvalidToken)?;

        drop(keys);

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<OidcClaims>(token, &decoding_key, &validation)
            .map_err(|e| {
                tracing::debug!("JWT validation failed: {e}");
                AuthError::InvalidToken
            })?;

        Ok(token_data.claims)
    }

    /// Refresh the JWKS keys from the provider.
    ///
    /// Call this periodically (e.g. every 15 min) to pick up key rotations.
    pub async fn refresh_keys(&self) -> Result<(), String> {
        let jwks: Jwks = reqwest::get(&self.jwks_uri)
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {e}"))?;

        let mut keys = self.keys.write().await;
        *keys = jwks.keys;
        tracing::info!("Refreshed JWKS keys");
        Ok(())
    }
}

fn find_key<'a>(keys: &'a [Jwk], kid: Option<&str>) -> Option<&'a Jwk> {
    keys.iter().find(|k| {
        k.kty == "RSA"
            && match (kid, k.kid.as_deref()) {
                (Some(want), Some(have)) => want == have,
                (None, _) => true, // No kid in header → use first RSA key.
                _ => false,
            }
    })
}

/// Authentication error returned to the client.
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    KeyNotFound,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Missing Authorization header")
            }
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::KeyNotFound => {
                (StatusCode::UNAUTHORIZED, "Signing key not found")
            }
        };

        (
            status,
            Json(serde_json::json!({
                "error": "unauthorized",
                "detail": msg
            })),
        )
            .into_response()
    }
}

/// Axum extractor that validates the JWT Bearer token and returns OIDC claims.
///
/// Usage in handlers:
/// ```ignore
/// async fn my_handler(AuthClaims(claims): AuthClaims) -> impl IntoResponse { ... }
/// ```
pub struct AuthClaims(pub OidcClaims);

impl<S: Send + Sync> FromRequestParts<S> for AuthClaims
where
    S: AsRef<Option<JwksCache>>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let cache = state
            .as_ref()
            .as_ref()
            .ok_or(AuthError::MissingToken)?;

        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MissingToken)?;

        let claims = cache.validate_token(token).await?;
        Ok(AuthClaims(claims))
    }
}
