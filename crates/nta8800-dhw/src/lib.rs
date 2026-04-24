//! # nta8800-dhw
//!
//! NTA 8800:2025+C1:2026 H.13 — warm tapwater (DHW: Domestic Hot Water).
//!
//! Rekent de **nettowarmtebehoefte** `Q_W;nd` om naar **Q_W;use** — het
//! eindenergiegebruik per energiedrager voor warmtapwaterbereiding, gedeeld
//! door de keten-rendementen:
//!
//! ```text
//! Q_W;use;mi = (Q_W;nd;mi − Q_W;rcd;mi) / (η_W;em × η_W;dis × η_W;gen)    [MJ]
//! ```
//!
//! waarbij `Q_W;rcd` de thermische bijdrage is van douchewaterwarmte­terugwinning
//! (DWTW, bijlage U) en vóór de keten-verliezen van de netto-vraag wordt
//! afgetrokken conform 13.1.1.2 ("De bijdrage van een eventueel DWTW-systeem
//! wordt afgetrokken van de benodigde warmte voor het afgiftesysteem voor warm
//! tapwater.").
//!
//! ## V1 scope (strict beperkt)
//!
//! **IN V1:**
//! - §13.2 nettowarmtebehoefte `Q_W;nd`:
//!   - woningbouw forfaitair 856 kWh/jaar per bewoner (formule 13.15,
//!     §13.2.3.1), aantal bewoners via A_g/N formule 13.16-13.18.
//!   - utiliteitsbouw forfaitair per gebruiksfunctie (tabel 13.1, formule 13.19):
//!     kantoorfunctie 1,4 – gezondheidszorg met bedgebied 15,3 kWh/m²·jaar.
//!   - user-supplied override beschikbaar voor beide categorieën.
//!   - maandverdeling op basis van t_mi/t_an (uren per maand ÷ 8760).
//! - §13.3 afgifterendement `η_W;em` (gesimplificeerd):
//!   - woningbouw: één waarde per systeem, default 0,65 (uittapleiding ~4 m
//!     keuken + ~3 m badruimte, tabel 13.2 berekening via formule 13.23).
//!   - utiliteitsbouw tabel 13.3: 1,0 bij ≤ 3 m, 0,8 bij > 3 m.
//!   - `Custom { efficiency }` voor user-supplied waarden.
//! - §13.4 distributierendement `η_W;dis` forfaitair (circulatieleiding-
//!   verliezen vereenvoudigd tot één factor; conversieverlies afleverset,
//!   13.4.2, staat buiten V1 scope).
//! - §13.8 opwekkingsrendement per generator-type [`DhwGenerationSystem`]:
//!   - HR-combi-ketel (gas, η_W;gen default 0,80 — lager dan CV door meer
//!     deellast bij kleine tappingen).
//!   - Elektrische boiler (electricity, η_W;gen default 0,90).
//!   - Warmtepomp tapwater (electricity, SCOP_W user-supplied).
//!   - Stadsverwarming (DistrictHeat, forfaitaire factor ≤ 1,0).
//! - §13.5 + bijlage U douche-WTW via [`DouchewtwRecovery`]: aftrek volgens
//!   formule 13.51, vereenvoudigd tot `Q_rcd = η × (C_W;nd;sh × Q_W;nd)` met
//!   vaste `C_W;nd;sh = 0,4` (typisch woning-aandeel douche in Q_W;nd) en
//!   f_prac = 1,0 × C_T × C_conf. User levert net thermisch rendement η.
//!
//! **NIET in V1 (V2 scope):**
//! - Bijlage T — tappatroon CDR 811/813/814 en Gaskeur-koppeling (detail-
//!   koppeling met warmtapwater-toestellen conform NEN-EN 13203-2 /
//!   NEN-EN 16147). V1 gebruikt forfaitair η_gen.
//! - Bijlage U — volledige DWTW-methodiek (C_W;sh;rcd;T, C_W;sh;rcd;conf,
//!   meerdere douches, bijlage U formule 13.53-13.57). V1 exposeert alleen
//!   netto thermisch rendement η.
//! - Bijlage W — booster-warmtepomp detail.
//! - §13.4 circulatieleidingen met lineair verlies × leidinglengte
//!   (formule 13.25 e.v.) + conversieverlies afleverset formule 13.24a.
//! - §13.5 warmteterugwinning uit riool.
//! - §13.6 voorraadvat-verliezen (Q_W;sto;ls, formule 13.58-13.59, S_sto;ls
//!   vaste waarden).
//! - §13.7 zonneboiler-bijdrage.
//! - Hulpenergie (formule 13.2, W_W;aux).
//! - Parallelle bedrijfswijze combi-toestel verwarming+tapwater
//!   (13.8.4.9.3).
//!
//! ## Publieke API
//!
//! - [`calculate_dhw`] — entry-point die een [`nta8800_model::Rekenzone`] en
//!   een [`DhwDemand`] (+ keten-componenten) omzet naar [`DhwResult`].
//! - [`DhwDemand`], [`DhwEmission`], [`DhwDistribution`],
//!   [`DhwGenerationSystem`], [`DouchewtwRecovery`] — invoer-modellen.
//! - [`DhwResult`] + [`DhwBreakdown`] — resultaat met audit-trace.
//! - [`EnergyCarrier`] — energiedrager voor downstream nEP-berekening.
//!
//! ## Energiedragers
//!
//! De eenheid van `Q_W;use` blijft altijd **MJ**, maar de betekenis hangt af
//! van de opwekker:
//!
//! | Generator | EnergyCarrier | Betekenis van `Q_W;use` |
//! |---|---|---|
//! | HR-combi-ketel | Gas | Gas-energie (onderwaarde) |
//! | Elektrische boiler | Electricity | Elektrische input-energie |
//! | Warmtepomp DHW | Electricity | Elektrische input-energie |
//! | Stadsverwarming | DistrictHeat | Warmte-input bij grensvlak gebouw |

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments gebruiken NTA 8800-symbolen (η_W;em, Q_W;nd, Q_W;use, etc.) die
// de backtick-heuristiek als false positive oppikt — consistent met
// demand/heating-crates.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_dhw;
pub use errors::{DhwCalcResult, DhwError};
pub use model::{
    DhwDemand, DhwDistribution, DhwEmission, DhwGenerationSystem, DouchewtwRecovery, EnergyCarrier,
};
pub use result::{DhwBreakdown, DhwResult};
