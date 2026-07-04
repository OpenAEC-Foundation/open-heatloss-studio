//! Norm-referenties voor de nta8800-core orchestratie.
//!
//! Volgt de isso51-core/isso53-core conventie: constanten die per keten-stap
//! naar het NTA 8800-hoofdstuk verwijzen dat de betreffende sub-crate
//! implementeert. De formule-detail-referenties zelf leven in de sub-crates
//! (zie `nta8800_model::references`); deze façade documenteert alleen de
//! keten-volgorde.

/// H.8 — transmissie warmteverlies (via `nta8800-transmission`).
pub const KETEN_TRANSMISSIE: &str = "NTA 8800:2025+C1:2026 H.8";

/// H.11 — ventilatie + infiltratie (via `nta8800-ventilation`).
pub const KETEN_VENTILATIE: &str = "NTA 8800:2025+C1:2026 H.11";

/// H.7 — maandelijkse warmte- en koudebehoefte (via `nta8800-demand`).
pub const KETEN_BEHOEFTE: &str = "NTA 8800:2025+C1:2026 H.7";

/// H.9 — verwarming: afgifte, distributie, opwekking, regeling
/// (via `nta8800-heating`).
pub const KETEN_VERWARMING: &str = "NTA 8800:2025+C1:2026 H.9";

/// H.10 — koeling (via `nta8800-cooling`).
pub const KETEN_KOELING: &str = "NTA 8800:2025+C1:2026 H.10";

/// H.13 — warm tapwater (via `nta8800-dhw`).
pub const KETEN_TAPWATER: &str = "NTA 8800:2025+C1:2026 H.13";

/// H.14 — verlichting, alleen utiliteitsfuncties (via `nta8800-lighting`).
pub const KETEN_VERLICHTING: &str = "NTA 8800:2025+C1:2026 H.14";

/// H.16 — PV-opbrengst (via `nta8800-pv`).
pub const KETEN_PV: &str = "NTA 8800:2025+C1:2026 H.16";

/// H.5 — EP-score, primaire energie en energielabel (via `nta8800-ep`).
pub const KETEN_EP: &str = "NTA 8800:2025+C1:2026 H.5 + bijlagen Z/AB";

/// §11.2.2 — norm-forfait benodigde luchtvolumestroom van buitenlucht
/// `q_V;ODA;req`, gebruikt als terugval wanneer geen luchtdebieten zijn
/// ingevoerd (formules 11.22 / 11.56 / 11.57 / 11.63 + tabel 11.8).
pub const FORFAIT_QV_ODA_REQ: &str = "NTA 8800:2025+C1:2026 §11.2.2";
