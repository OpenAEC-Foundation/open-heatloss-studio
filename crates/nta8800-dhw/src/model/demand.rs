//! Nettowarmtebehoefte warm tapwater `Q_W;nd` — forfaitair of user-supplied.
//!
//! NTA 8800:2025+C1:2026 §13.2 onderscheidt twee categorieën:
//!
//! - **Woningbouw** (§13.2.2.1, formule 13.15): forfaitaire behoefte van
//!   **856 kWh/jaar per bewoner** (§13.2.3.1). Aantal bewoners volgens
//!   formules 13.16-13.18 op basis van A_g / N_woon.
//! - **Utiliteitsbouw** (§13.2.2.2, formule 13.19): forfaitaire behoefte
//!   per gebruiksfunctie × A_g, uit tabel 13.1 (kWh/m²·jaar).
//!
//! V1 verdeelt de jaarlijkse behoefte **gelijkmatig** over de 12 maanden
//! (t_mi / t_an ≈ 1/12). De norm gebruikt formeel t_mi/t_an met
//! maandlengten van 17.2, maar §13.3.2.1 OPMERKING erkent dat er bij de
//! gegeven getalswaarden sprake is van "schijnnauwkeurigheid" als per
//! maand uitgerekend wordt — dezelfde jaarwaarde geldt voor elke maand.
//! V1 gaat consistent uit van gelijke maandverdeling.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::{Area, Energy};
use nta8800_model::zoning::UsageFunction;

use crate::errors::{DhwCalcResult, DhwError};

/// 1 kWh = 3,6 MJ — conversie NTA 8800 (kWh) → `Energy` (MJ).
const MJ_PER_KWH: f64 = 3.6;

/// §13.2.3.1 — Specifieke warmtebehoefte warm tapwater categorie woningbouw:
/// 856 kWh per bewoner per jaar.
const Q_SPEC_P_KWH_PER_YEAR: f64 = 856.0;

/// Nettowarmtebehoefte warm tapwater `Q_W;nd` per maand, in MJ.
///
/// De onderliggende opslag is een [`MonthlyProfile<Energy>`] zodat latere
/// varianten (bijlage T tappatronen) de maandverdeling kunnen differentiëren
/// zonder publieke API-breuk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DhwDemand {
    /// Maandelijkse nettowarmtebehoefte `Q_W;nd;mi` in MJ.
    pub monthly_demand: MonthlyProfile<Energy>,
}

impl DhwDemand {
    /// Forfaitaire nettowarmtebehoefte voor **woningbouw** conform §13.2.3.1.
    ///
    /// Gebruikt de specifieke behoefte van 856 kWh/jaar per bewoner en bepaalt
    /// het aantal bewoners N_P volgens formules 13.16-13.18 op basis van de
    /// gebruiksoppervlakte per woonfunctie. Voor één woonfunctie geldt
    /// N_woon = 1 en A_g/N_woon = A_g.
    ///
    /// # Argumenten
    ///
    /// - `floor_area_m2` — gebruiksoppervlakte A_g van de woonfunctie in m².
    ///
    /// # Errors
    ///
    /// [`DhwError::InvalidFloorArea`] als `floor_area_m2 ≤ 0` of niet-eindig.
    ///
    /// # Referentie
    ///
    /// NTA 8800:2025+C1:2026, §13.2.2.1 (formule 13.15), §13.2.3.1.
    pub fn forfaitair_woningbouw(floor_area_m2: Area) -> DhwCalcResult<Self> {
        if !floor_area_m2.is_finite() || floor_area_m2 <= 0.0 {
            return Err(DhwError::InvalidFloorArea {
                area: floor_area_m2,
            });
        }
        let n_persons = residents_per_dwelling(floor_area_m2);
        // Q_W;nd;an = Q_spec;p × N_P (N_woon = 1 voor enkelvoudige woonfunctie)
        let annual_kwh = Q_SPEC_P_KWH_PER_YEAR * n_persons;
        let annual_mj = annual_kwh * MJ_PER_KWH;
        Ok(Self::from_annual_even(annual_mj))
    }

    /// Forfaitaire nettowarmtebehoefte voor **utiliteitsbouw** conform
    /// §13.2.3.2 tabel 13.1.
    ///
    /// Gebruikt de specifieke behoefte per gebruiksfunctie (kWh/m²·jaar) ×
    /// gebruiksoppervlakte. Woonfunctie valt buiten tabel 13.1; gebruik
    /// [`DhwDemand::forfaitair_woningbouw`] in plaats.
    ///
    /// # Argumenten
    ///
    /// - `usage` — gebruiksfunctie.
    /// - `floor_area_m2` — A_g in m².
    ///
    /// # Errors
    ///
    /// - [`DhwError::InvalidFloorArea`] als `floor_area_m2 ≤ 0`.
    /// - [`DhwError::Model`] wrapper rond een validatie-fout wanneer
    ///   `usage == Woonfunctie` — deze moet via
    ///   [`DhwDemand::forfaitair_woningbouw`].
    ///
    /// # Referentie
    ///
    /// NTA 8800:2025+C1:2026, §13.2.2.2 (formule 13.19), tabel 13.1.
    pub fn forfaitair_utiliteit(usage: UsageFunction, floor_area_m2: Area) -> DhwCalcResult<Self> {
        if !floor_area_m2.is_finite() || floor_area_m2 <= 0.0 {
            return Err(DhwError::InvalidFloorArea {
                area: floor_area_m2,
            });
        }
        let spec_kwh_m2 = utility_specific_demand_kwh_m2_year(usage)?;
        let annual_kwh = spec_kwh_m2 * floor_area_m2;
        let annual_mj = annual_kwh * MJ_PER_KWH;
        Ok(Self::from_annual_even(annual_mj))
    }

    /// Maak een `DhwDemand` direct vanuit een user-supplied maandprofiel
    /// (in MJ). Voor situaties waarbij een kwaliteitsverklaring of meet-data
    /// een afwijkende tapwaterbehoefte oplevert.
    #[must_use]
    pub const fn from_monthly(profile: MonthlyProfile<Energy>) -> Self {
        Self {
            monthly_demand: profile,
        }
    }

    /// Maak een `DhwDemand` vanuit een user-supplied **jaarlijkse** behoefte
    /// (in MJ), gelijkmatig verdeeld over de 12 maanden.
    ///
    /// # Errors
    ///
    /// Geen — negatieve jaarwaarden worden als 0 geïnterpreteerd (geen fysische
    /// betekenis, maar V1 behandelt dit grafisch).
    #[must_use]
    pub fn from_annual_even(annual_mj: Energy) -> Self {
        let monthly = annual_mj / 12.0;
        Self {
            monthly_demand: MonthlyProfile::from_constant(monthly),
        }
    }

    /// Jaarlijks totaal Q_W;nd;an in MJ.
    #[must_use]
    pub fn annual(&self) -> Energy {
        self.monthly_demand.as_array().iter().sum()
    }
}

/// Aantal bewoners N_P per woonfunctie volgens formules 13.16-13.18.
///
/// Geeft een **gladde** functie van A_g/N_woon — geen stap-functie:
/// - A/N ≤ 30 m²: N_P = 1
/// - 30 < A/N ≤ 100 m²: N_P = 2,28 − 1,28 × (100 − A/N) / 70
///   ⇒ bij A/N = 100: N_P = 2,28; bij A/N = 30: N_P = 1,00
/// - A/N > 100 m²: N_P = 1,28 + 0,01 × A/N
///   ⇒ bij A/N = 100: N_P = 2,28; bij A/N = 200: N_P = 3,28
///
/// Voor enkelvoudige woonfunctie N_woon = 1 geldt A/N = A_g.
fn residents_per_dwelling(floor_area_per_dwelling: f64) -> f64 {
    let a = floor_area_per_dwelling;
    if a <= 30.0 {
        1.0
    } else if a <= 100.0 {
        2.28 - 1.28 * (100.0 - a) / 70.0
    } else {
        1.28 + 0.01 * a
    }
}

/// Specifieke nettowarmtebehoefte warm tapwater per gebruiksfunctie voor
/// utiliteitsbouw — tabel 13.1 NTA 8800:2025+C1:2026, in kWh/m² per jaar.
///
/// # Errors
///
/// Retourneert een fout voor `UsageFunction::Woonfunctie` (niet in tabel 13.1;
/// gebruik [`DhwDemand::forfaitair_woningbouw`]) en voor
/// `UsageFunction::Industriefunctie` (expliciet **niet** in tabel 13.1; norm
/// geeft hiervoor geen forfaitaire waarde, engineering-invulling nodig).
// Bewust expliciete arms per gebruiksfunctie ondanks numerieke overlap: één-
// op-één mapping op tabel 13.1 voor audit-traceability + toekomstige norm-
// differentiatie (bv. kinderopvang krijgt een eigen waarde in latere release).
#[allow(clippy::match_same_arms)]
fn utility_specific_demand_kwh_m2_year(usage: UsageFunction) -> DhwCalcResult<f64> {
    // NTA 8800:2025+C1:2026 tabel 13.1 — "De jaarlijkse specifieke
    // nettowarmtebehoefte voor warm tapwater, Q_W;nd;spec (vaste waarden) per
    // gebruiksfunctie", in kWh/m² per jaar.
    //
    // Bijeenkomst (kinderopvang + overig) zijn beide 2,8. Gezondheidszorg
    // wordt in V1 als **met bedgebied** 15,3 kWh/m²·jaar genomen — dit is de
    // zwaarste variant; projecten met kantoor-achtige zorg (polikliniek)
    // moeten via `from_annual_even` of `from_monthly` met 2,8 × A_g
    // overrulen. Zie lessons learned §"Rapportage design principe".
    match usage {
        UsageFunction::Woonfunctie => {
            Err(DhwError::Model(nta8800_model::ModelError::InvalidInput {
                context: "DhwDemand::forfaitair_utiliteit usage".into(),
                reason: "UsageFunction::Woonfunctie valt buiten tabel 13.1; gebruik \
                     DhwDemand::forfaitair_woningbouw"
                    .into(),
            }))
        }
        UsageFunction::Industriefunctie => {
            // Tabel 13.1 vermeldt geen industriefunctie. De norm geeft hier
            // geen forfaitaire waarde — per rekenzone moet een engineering-
            // waarde worden opgegeven via `DhwDemand::from_annual_even` /
            // `DhwDemand::from_monthly`. V1 valt bewust niet terug op een
            // default om verborgen engineering-aannames te voorkomen (zie
            // lessons learned §"Rapportage design principe").
            Err(DhwError::Model(nta8800_model::ModelError::InvalidInput {
                context: "DhwDemand::forfaitair_utiliteit usage".into(),
                reason: "UsageFunction::Industriefunctie heeft geen forfaitaire waarde in \
                         tabel 13.1; geef expliciet Q_W;nd via DhwDemand::from_annual_even"
                    .into(),
            }))
        }
        UsageFunction::Bijeenkomstfunctie => Ok(2.8),
        UsageFunction::Celfunctie => Ok(4.2),
        UsageFunction::Gezondheidszorgfunctie => Ok(15.3),
        UsageFunction::Kantoorfunctie => Ok(1.4),
        UsageFunction::Logiesfunctie => Ok(12.5),
        UsageFunction::Onderwijsfunctie => Ok(1.4),
        UsageFunction::Sportfunctie => Ok(12.5),
        UsageFunction::Winkelfunctie => Ok(1.4),
        UsageFunction::OverigeGebruiksfunctie => {
            // Tabel 13.1 kent geen "overig" categorie. Conservatieve fallback:
            // 2,8 kWh/m²·jaar (zelfde als bijeenkomst/overig) — documentair
            // gemarkeerd als engineering-default. Projecten met specifieke
            // invulling moeten `from_annual_even` gebruiken.
            Ok(2.8)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nta8800_model::time::Month;

    #[test]
    fn residents_tiny_apartment_is_one() {
        assert!((residents_per_dwelling(25.0) - 1.0).abs() < 1e-12);
        assert!((residents_per_dwelling(30.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn residents_medium_apartment_smooth() {
        // A = 65 m² ⇒ (100-65)/70 = 0,5 ⇒ N = 2,28 - 1,28 × 0,5 = 1,64
        assert_relative_eq!(residents_per_dwelling(65.0), 1.64, max_relative = 1e-9);
    }

    #[test]
    fn residents_large_house_linear_growth() {
        // A = 150 ⇒ N = 1,28 + 1,5 = 2,78
        assert_relative_eq!(residents_per_dwelling(150.0), 2.78, max_relative = 1e-9);
    }

    #[test]
    fn residents_boundary_100_continuous() {
        // Beide branches moeten 2,28 opleveren bij A = 100.
        let low = residents_per_dwelling(100.0);
        let high_eps = residents_per_dwelling(100.0 + 1e-9);
        assert!(
            (low - 2.28).abs() < 1e-9,
            "branch ≤100 geeft {low}, verwacht 2,28"
        );
        assert!(
            (high_eps - 2.28).abs() < 1e-7,
            "branch >100 geeft {high_eps}, verwacht 2,28 (continuïteit)"
        );
    }

    #[test]
    fn forfaitair_woningbouw_100m2_plausible() {
        // 100 m² woning: N_P = 2,28 bewoners
        // Q_W;nd = 856 × 2,28 = 1.951,68 kWh/jaar = 7.026 MJ/jaar
        let demand = DhwDemand::forfaitair_woningbouw(100.0).unwrap();
        let annual = demand.annual();
        assert_relative_eq!(annual, 856.0 * 2.28 * MJ_PER_KWH, max_relative = 1e-9);
        // Plausibiliteit: 7 GJ/jaar past bij norm-range 10-15 GJ inclusief verliezen
        // (scope is Q_W;nd, zonder verliezen-chain, dus wat lager).
        assert!(annual > 6_000.0 && annual < 8_000.0);
    }

    #[test]
    fn forfaitair_woningbouw_monthly_even() {
        let demand = DhwDemand::forfaitair_woningbouw(120.0).unwrap();
        let jan = demand.monthly_demand[Month::Januari];
        let dec = demand.monthly_demand[Month::December];
        assert_relative_eq!(jan, dec, max_relative = 1e-9);
        assert_relative_eq!(jan * 12.0, demand.annual(), max_relative = 1e-9);
    }

    #[test]
    fn forfaitair_woningbouw_rejects_zero_area() {
        let err = DhwDemand::forfaitair_woningbouw(0.0).unwrap_err();
        assert!(matches!(err, DhwError::InvalidFloorArea { .. }));
    }

    #[test]
    fn forfaitair_woningbouw_rejects_nan() {
        let err = DhwDemand::forfaitair_woningbouw(f64::NAN).unwrap_err();
        assert!(matches!(err, DhwError::InvalidFloorArea { .. }));
    }

    #[test]
    fn forfaitair_utiliteit_kantoor() {
        // Kantoor 1,4 kWh/m²·jaar × 500 m² = 700 kWh/jaar = 2.520 MJ/jaar
        let demand = DhwDemand::forfaitair_utiliteit(UsageFunction::Kantoorfunctie, 500.0).unwrap();
        assert_relative_eq!(demand.annual(), 700.0 * MJ_PER_KWH, max_relative = 1e-9);
    }

    #[test]
    fn forfaitair_utiliteit_gezondheidszorg_highest() {
        // Gezondheidszorg met bedgebied = hoogste kWh/m²·jaar
        let zorg = DhwDemand::forfaitair_utiliteit(UsageFunction::Gezondheidszorgfunctie, 1_000.0)
            .unwrap();
        let kantoor =
            DhwDemand::forfaitair_utiliteit(UsageFunction::Kantoorfunctie, 1_000.0).unwrap();
        assert!(zorg.annual() > kantoor.annual() * 10.0);
    }

    #[test]
    fn forfaitair_utiliteit_woning_rejected() {
        let err = DhwDemand::forfaitair_utiliteit(UsageFunction::Woonfunctie, 100.0).unwrap_err();
        assert!(matches!(err, DhwError::Model(_)));
    }

    #[test]
    fn forfaitair_utiliteit_industrie_rejected() {
        let err =
            DhwDemand::forfaitair_utiliteit(UsageFunction::Industriefunctie, 500.0).unwrap_err();
        assert!(matches!(err, DhwError::Model(_)));
    }

    #[test]
    fn from_annual_even_distributes_monthly() {
        let demand = DhwDemand::from_annual_even(12_000.0);
        for m in Month::all() {
            assert_relative_eq!(demand.monthly_demand[m], 1_000.0, max_relative = 1e-12);
        }
    }

    #[test]
    fn from_monthly_preserves_profile() {
        let profile = MonthlyProfile::new([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ]);
        let demand = DhwDemand::from_monthly(profile.clone());
        for m in Month::all() {
            assert_relative_eq!(demand.monthly_demand[m], profile[m], max_relative = 1e-12);
        }
        assert_relative_eq!(demand.annual(), 78.0, max_relative = 1e-12);
    }

    #[test]
    fn serde_round_trip() {
        let demand = DhwDemand::forfaitair_woningbouw(100.0).unwrap();
        let json = serde_json::to_string(&demand).unwrap();
        let back: DhwDemand = serde_json::from_str(&json).unwrap();
        assert_eq!(demand, back);
    }
}
