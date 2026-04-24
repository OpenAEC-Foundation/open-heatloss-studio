//! # nta8800-lighting
//!
//! NTA 8800:2025+C1:2026 H.14 — verlichting (utiliteitsfuncties).
//!
//! Berekent het eindenergiegebruik voor verlichting `W_L;use` per maand per
//! rekenzone. Uitvoer is altijd in [`EnergyCarrier::Electricity`] want
//! verlichting wordt volgens NTA 8800 H.14 altijd elektrisch gevoed.
//!
//! ## V1 scope (strict beperkt)
//!
//! **IN V1:**
//! - §14.2.2 — energiebehoefte utiliteitsfuncties via één lumped formule:
//!
//!   ```text
//!   W_L;use;mi = P_n × F_u × F_d × F_c × A_f × t_mi × 3600 / 10^6   [MJ]
//!   ```
//!
//!   Dit is een maandelijkse decompositie van §14.2.2 formule (14.7). Het
//!   origineel rekent jaarlijks met twee termen (t_D en t_N) — V1 slaat die
//!   samen in één bezettingsfactor `F_u` en verdeelt evenredig naar de
//!   kalenderuren per maand.
//!
//! - §14.3.4 tabel 14.3 — forfaitair `P_n;spec` in W/m²:
//!   - 16 W/m² voor bijeenkomst / kantoor / onderwijs / sport / GZ-zonder-bed
//!   - 17 W/m² voor cel / logies / GZ-met-bed
//!   - 30 W/m² voor winkelfunctie
//!
//! - §14.5.1 — `F_o;D = F_o;N = 1,0` (geen aanwezigheidsdetectie) en
//!   §14.4 tabel 14.4 — `F_c = 1,0` (geen nieuwwaarde-compensatie) als default.
//!
//! **NIET in V1 (V2 scope):**
//! - §14.2.1 woonfunctie — voor utiliteit-scope projecten wordt
//!   verlichting in de woonfunctie **niet** in de nEP-indicator meegeteld
//!   (W_L;spec = 0 kWh/m²). De norm laat het wel toe voor maatwerk; V2 voegt
//!   een expliciete woonfunctie-modus toe.
//! - §14.3.3 parasitair vermogen (noodverlichting accu-laders, stand-by
//!   besturing) — aparte post `W_P`, V2.
//! - §14.5.2 aanwezigheid-afhankelijke regelingen met automatische detectie
//!   — V2 tabel 14.5 / 14.6.
//! - §14.6 daglichtafhankelijkheid via bijlage Y daglichttoetreding voor
//!   hellende ramen — V1 laat `F_d` als user-supplied scalar (default 1,0);
//!   volledige bijlage Y is V2.
//! - §14.3.2 werkelijk geïnstalleerd vermogen via armatuur-sommatie + tabel
//!   14.2 voorschakelapparatuur — V1 gebruikt alleen forfaitaire `P_n;spec`
//!   óf user-supplied scalar.
//! - Nachtelijke correcties weekend / feestdagen — V2.
//!
//! ## Publieke API
//!
//! - [`calculate_lighting`] — entry-point, maakt [`LightingResult`] voor één
//!   [`Rekenzone`](nta8800_model::zoning::Rekenzone) + [`LightingSystem`].
//! - [`LightingSystem`] — invoer-model met vier scalars (`P_n`, `F_u`, `F_d`,
//!   `F_c`) plus forfaitair-constructor per [`UsageFunction`].
//! - [`LightingResult`] — maandprofiel + jaartotaal + metadata.
//! - [`EnergyCarrier`] — altijd elektrisch voor verlichting.
//!
//! ## Energiedrager
//!
//! Verlichting is volgens NTA 8800 H.14 altijd elektrisch. De uitvoer
//! [`LightingResult::energy_carrier`] is een constante
//! [`EnergyCarrier::Electricity`] — opgenomen voor API-consistentie met
//! andere installatie-crates en voor downstream `nEP`-berekening.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments gebruiken NTA 8800-symbolen (W_L;use, F_o;D, etc.) die de
// backtick-heuristiek als false positive oppikt — consistent met de andere
// nta8800-* crates.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_lighting;
pub use errors::{LightingCalcResult, LightingError};
pub use model::{EnergyCarrier, LightingSystem};
pub use result::LightingResult;
