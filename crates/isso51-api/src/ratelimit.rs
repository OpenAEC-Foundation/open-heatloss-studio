//! Lichtgewicht, dependency-vrije per-IP rate limiter voor de publieke
//! reken-routes (`/calculate`, `/calculate_v2`, `/calculate/ifcx`,
//! `/import/thermal`, `/cooling/simplified`, `/tojuli/calculate`).
//!
//! Deze routes hebben bewust geen authenticatie (publieke reken-API), maar zijn
//! CPU-intensief. Zonder begrenzing kan één client de server met rekenverzoeken
//! overspoelen. We hanteren een ruime vaste-venster-limiet per client-IP
//! (default 30 requests/minuut, override via `COMPUTE_RATE_LIMIT_PER_MIN`;
//! `0` schakelt de limiet uit — handig voor lokale ontwikkeling).
//!
//! **IP-extractie:** achter de Caddy reverse-proxy komt het echte client-IP
//! binnen via `X-Forwarded-For`. Caddy *appendt* het directe peer-IP rechts,
//! dus we nemen de meest rechtse entry — die is door onze eigen proxy gezet en
//! niet door de client te spoofen (een client kan alleen entries links
//! toevoegen). Ontbreekt de header (directe toegang / lokale dev), dan valt
//! alles in één gedeelde bucket (`"unknown"`); acceptabel voor een
//! compute-DoS-rem, en dat is precies de situatie zonder proxy ervoor.
//!
//! **Bewuste beperking:** een gedistribueerde aanval vanaf veel IP's of met
//! geroteerde peers omzeilt een per-IP-limiet — dit is defense-in-depth tegen
//! naïef misbruik, geen volwaardige DDoS-bescherming.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Boven dit aantal bijgehouden buckets ruimen we verlopen entries op, zodat
/// het geheugen niet onbegrensd groeit bij veel unieke IP's.
const PRUNE_THRESHOLD: usize = 10_000;

/// Vaste-venster teller per client-sleutel.
#[derive(Clone, Copy)]
struct Window {
    count: u32,
    reset_at: Instant,
}

/// Deelbare, thread-veilige per-IP rate limiter (vast venster).
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, Window>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    /// Bouw uit env: `COMPUTE_RATE_LIMIT_PER_MIN` (default 30). Een waarde van
    /// `0` schakelt de limiet uit (elk verzoek toegestaan).
    pub fn from_env() -> Self {
        let max = std::env::var("COMPUTE_RATE_LIMIT_PER_MIN")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(30);
        Self::new(max, Duration::from_secs(60))
    }

    /// Registreer een hit voor `key` op tijdstip `now`. Retourneert
    /// `Some(retry_after_secs)` als de limiet overschreden is, anders `None`.
    /// `now` is een parameter zodat de venster-logica deterministisch getest
    /// kan worden (geen echte klok nodig).
    fn check(&self, key: &str, now: Instant) -> Option<u64> {
        if self.max_requests == 0 {
            return None; // limiet uitgeschakeld
        }
        let mut map = self.inner.lock().expect("rate limiter mutex poisoned");

        if map.len() > PRUNE_THRESHOLD {
            map.retain(|_, w| w.reset_at > now);
        }

        let window = self.window;
        let entry = map.entry(key.to_owned()).or_insert(Window {
            count: 0,
            reset_at: now + window,
        });
        if now >= entry.reset_at {
            entry.count = 0;
            entry.reset_at = now + window;
        }
        entry.count += 1;
        if entry.count > self.max_requests {
            Some(entry.reset_at.saturating_duration_since(now).as_secs() + 1)
        } else {
            None
        }
    }
}

/// Bepaal de client-sleutel: meest rechtse `X-Forwarded-For`-entry (door onze
/// eigen proxy gezet), anders `X-Real-IP`, anders `"unknown"`.
fn client_key(headers: &HeaderMap) -> String {
    if let Some(xff) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(last) = xff.split(',').map(str::trim).rfind(|s| !s.is_empty()) {
            return last.to_owned();
        }
    }
    if let Some(real) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        let real = real.trim();
        if !real.is_empty() {
            return real.to_owned();
        }
    }
    "unknown".to_owned()
}

/// Axum-middleware: rate-limit op basis van client-IP. Bij overschrijding
/// `429 Too Many Requests` met een `Retry-After`-header en een `detail`-body
/// (zelfde JSON-vorm als de overige API-fouten).
pub async fn rate_limit(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    let key = client_key(req.headers());
    if let Some(retry_after) = limiter.check(&key, Instant::now()) {
        let mut resp = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "detail": "Te veel rekenverzoeken — probeer het over enige tijd opnieuw.",
                "type": "RateLimited",
            })),
        )
            .into_response();
        if let Ok(val) = retry_after.to_string().parse() {
            resp.headers_mut().insert("retry-after", val);
        }
        return resp;
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staat_tot_de_limiet_toe_en_weigert_daarna() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        let t0 = Instant::now();
        assert_eq!(rl.check("1.2.3.4", t0), None);
        assert_eq!(rl.check("1.2.3.4", t0), None);
        assert_eq!(rl.check("1.2.3.4", t0), None);
        let retry = rl.check("1.2.3.4", t0).expect("4e verzoek moet geweigerd worden");
        assert!(retry >= 1, "retry-after moet minstens 1 seconde zijn");
    }

    #[test]
    fn telt_per_ip_apart() {
        let rl = RateLimiter::new(1, Duration::from_secs(60));
        let t0 = Instant::now();
        assert_eq!(rl.check("a", t0), None);
        assert!(rl.check("a", t0).is_some());
        // Ander IP heeft een eigen bucket.
        assert_eq!(rl.check("b", t0), None);
    }

    #[test]
    fn reset_na_het_venster() {
        let rl = RateLimiter::new(1, Duration::from_secs(60));
        let t0 = Instant::now();
        assert_eq!(rl.check("x", t0), None);
        assert!(rl.check("x", t0).is_some());
        // Na het venster telt de bucket opnieuw vanaf 0.
        let later = t0 + Duration::from_secs(61);
        assert_eq!(rl.check("x", later), None);
    }

    #[test]
    fn limiet_nul_schakelt_uit() {
        let rl = RateLimiter::new(0, Duration::from_secs(60));
        let t0 = Instant::now();
        for _ in 0..1000 {
            assert_eq!(rl.check("x", t0), None);
        }
    }

    #[test]
    fn client_key_neemt_meest_rechtse_xff() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "9.9.9.9, 10.0.0.5".parse().unwrap());
        assert_eq!(client_key(&h), "10.0.0.5");
    }

    #[test]
    fn client_key_valt_terug_op_x_real_ip_dan_unknown() {
        let mut h = HeaderMap::new();
        h.insert("x-real-ip", "8.8.8.8".parse().unwrap());
        assert_eq!(client_key(&h), "8.8.8.8");
        assert_eq!(client_key(&HeaderMap::new()), "unknown");
    }
}
