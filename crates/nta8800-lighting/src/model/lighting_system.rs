//! [`LightingSystem`] — vier-scalar invoer-model voor §14.2.2.
//!
//! V1 bundelt de hele H.14 methodiek tot vier factoren:
//!
//! ```text
//! W_L;use;mi = P_n × F_u × F_d × F_c × A_f × t_mi × 3600 / 10^6   [MJ]
//! ```
//!
//! Zie [`crate`] module-doc voor de afleiding vanuit formule (14.7).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::zoning::UsageFunction;

use crate::errors::{LightingCalcResult, LightingError};

/// Energiedrager-annotatie voor `W_L;use`.
///
/// Verlichting is in NTA 8800 H.14 altijd elektrisch; deze enum heeft
/// bewust maar één variant, opgenomen voor API-consistentie met andere
/// installatie-crates ([`nta8800_heating`](https://docs.rs) etc.) en
/// downstream nEP-berekening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCarrier {
    /// Elektriciteit. Verlichting is altijd elektrisch.
    Electricity,
}

/// Verlichtings-systeem voor één rekenzone (V1 scope).
///
/// Vier scalars die samen formule (14.7) dekken via de lumped uitdrukking:
///
/// ```text
/// W_L;use = P_n × F_u × F_d × F_c × A × t_year × 3,6 / 10^3   [MJ/jaar]
/// ```
///
/// met `t_year = 8760 h` (kalenderuren). De maandverdeling gebeurt
/// evenredig op basis van kalenderuren per maand.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LightingSystem {
    /// Geïnstalleerd specifiek vermogen `P_n` in W/m².
    ///
    /// Bron: NTA 8800 §14.3.4 tabel 14.3 (forfaitair) of §14.3.2 werkelijk
    /// geïnstalleerd vermogen. V1 gebruikt enkel één scalar per zone.
    pub installed_power_w_per_m2: f64,

    /// Bezettingsfactor `F_u` (dimensieloos, `[0, 1]`).
    ///
    /// V1-lumped equivalent van NTA 8800 §14.5 `F_o;D` en `F_o;N` plus de
    /// brandduren `t_D` / `t_N` uit tabel 14.1:
    ///
    /// ```text
    /// F_u = (t_D × F_o;D × F_D + t_N × F_o;N) / 8760
    /// ```
    ///
    /// Voor een kantoor zonder detectie, zonder daglichtregeling:
    /// `F_u = (2200 × 1 + 300 × 1) / 8760 ≈ 0,285`.
    ///
    /// V1-forfaitair zet `F_D = 1` en `F_o;{D,N} = 1`; daglicht staat in
    /// [`Self::daylight_factor`].
    pub utilization_factor: f64,

    /// Daglichtcorrectiefactor `F_d` (dimensieloos, `[0, 1]`).
    ///
    /// V1 user-supplied scalar. NTA 8800 §14.6 + bijlage Y geven een
    /// gedetailleerde methodiek voor verticale + hellende ramen; V1 laat
    /// de zonering- en dakhelling-analyse aan de gebruiker. Richtwaarden:
    ///
    /// | Situatie | F_d |
    /// |---|---|
    /// | Geen daglichtregeling | 1,0 |
    /// | Continu dimbare daglichtregeling, goede daglichttoetreding | 0,6 |
    /// | Aan/uit daglichtregeling | 0,85 |
    pub daylight_factor: f64,

    /// Nieuwwaarde-compensatiefactor `F_c` (dimensieloos, `[0, 1]`).
    ///
    /// Bron: NTA 8800 §14.4 formule (14.15) + tabel 14.4:
    ///
    /// ```text
    /// F_c = 1 − (1/2) × F_cc × (1 − MF)
    /// ```
    ///
    /// met `F_cc = 1` altijd. Voor MF geldt:
    ///
    /// | Systeem | MF | F_c resultaat |
    /// |---|---|---|
    /// | Geen nieuwwaarde-compensatie / onbekend | 1,0 | 1,0 |
    /// | Lineair fluorescentie met compensatie | 0,8 | 0,9 |
    /// | LED (L80) met compensatie | 0,7 | 0,85 |
    pub control_factor: f64,
}

impl LightingSystem {
    /// Construct een `LightingSystem` met expliciete parameters en valideer.
    ///
    /// # Errors
    ///
    /// - [`LightingError::InvalidInstalledPower`] als `p_n < 0` of niet-eindig.
    /// - [`LightingError::InvalidFactor`] als een van `F_u / F_d / F_c` buiten
    ///   `[0, 1]` valt of niet-eindig is.
    pub fn new(
        installed_power_w_per_m2: f64,
        utilization_factor: f64,
        daylight_factor: f64,
        control_factor: f64,
    ) -> LightingCalcResult<Self> {
        let s = Self {
            installed_power_w_per_m2,
            utilization_factor,
            daylight_factor,
            control_factor,
        };
        s.validate()?;
        Ok(s)
    }

    /// Forfaitair `LightingSystem` voor een gegeven [`UsageFunction`].
    ///
    /// Bron: NTA 8800:2025+C1:2026 **pg 632-633** — tabel 14.3 (P_n;spec
    /// W/m²) gecombineerd met **pg 629** tabel 14.1 (brandduren t_D / t_N).
    ///
    /// Forfaitaire assumpties:
    /// - `F_o;D = F_o;N = 1,0` (§14.5.1, geen detectie)
    /// - `F_D = 1,0` (§14.6, geen daglichtregeling → F_d in struct = 1,0)
    /// - `F_C = 1,0` (§14.4 tabel 14.4, onbekende / geen compensatie)
    /// - `F_u = (t_D + t_N) / 8760` (lumped jaar-bezetting)
    ///
    /// Speciaal geval: [`UsageFunction::Woonfunctie`] en
    /// [`UsageFunction::Industriefunctie`] / [`UsageFunction::OverigeGebruiksfunctie`]
    /// staan niet in tabel 14.3. Voor V1 mappen we:
    ///
    /// | Gebruiksfunctie | P_n [W/m²] | t_D [h] | t_N [h] |
    /// |---|---|---|---|
    /// | Woonfunctie | 0,57 (≙ richtwaarde 5 kWh/m²·jr) | 8760 | 0 |
    /// | Industrie / Overige | 16 | 2200 | 300 |
    ///
    /// Zie [`forfaitair_details`](Self::forfaitair_details) voor de ruwe
    /// tabelwaarden voordat ze gecombineerd worden tot `F_u`.
    #[must_use]
    pub fn forfaitair(usage: UsageFunction) -> Self {
        let (p_n, t_d, t_n) = Self::forfaitair_details(usage);
        // F_u = (t_D × 1 + t_N × 1) / 8760. Woonfunctie kiest t_D = 8760
        // zodat het jaartotaal gelijk is aan P_n × A × 8760 × 3,6e-3 MJ,
        // d.w.z. P_n · A · 8760 / 1000 kWh — de norm-forfaitaire vorm.
        let f_u = (t_d + t_n) / 8760.0;
        Self {
            installed_power_w_per_m2: p_n,
            utilization_factor: f_u,
            daylight_factor: 1.0,
            control_factor: 1.0,
        }
    }

    /// Ruwe forfaitaire tabelwaarden `(P_n;spec, t_D, t_N)` per gebruiksfunctie.
    ///
    /// Gesplitst van [`forfaitair`](Self::forfaitair) om tests en rapportage
    /// de brontabel-waarden te laten inzien zonder de lumped `F_u` eruit
    /// te reverse-engineeren.
    #[must_use]
    pub const fn forfaitair_details(usage: UsageFunction) -> (f64, f64, f64) {
        match usage {
            // §14.2.1 woonfunctie — richtwaarde W_L;spec = 5 kWh/m²·jr
            // (opmerking bij formule 14.2). Omgerekend: P_n = 5 × 1000 / 8760
            // ≈ 0,5708 W/m² bij t = 8760 h, F_u = 1.
            UsageFunction::Woonfunctie => (5000.0 / 8760.0, 8760.0, 0.0),
            // Tabel 14.3 kolom 16 W/m²; tabel 14.1 brandduren 2200 / 300.
            // Gezondheidszorgfunctie "anders dan met bedgebied" valt ook onder
            // 16 W/m²; de UsageFunction-enum kent geen bed-variant, V1 kiest
            // de ondergrens als conservatieve default. Industrie en overige
            // staan niet in tabel 14.3 — kantoor-profiel als fallback.
            UsageFunction::Bijeenkomstfunctie
            | UsageFunction::Kantoorfunctie
            | UsageFunction::Gezondheidszorgfunctie
            | UsageFunction::Industriefunctie
            | UsageFunction::OverigeGebruiksfunctie => (16.0, 2200.0, 300.0),
            UsageFunction::Onderwijsfunctie => (16.0, 1600.0, 300.0),
            UsageFunction::Sportfunctie => (16.0, 2200.0, 800.0),
            // Tabel 14.3 kolom 17 W/m²; tabel 14.1 brandduren 4000 / 1000.
            UsageFunction::Celfunctie | UsageFunction::Logiesfunctie => (17.0, 4000.0, 1000.0),
            // Tabel 14.3 kolom 30 W/m²; tabel 14.1 brandduren 2700 / 400.
            UsageFunction::Winkelfunctie => (30.0, 2700.0, 400.0),
        }
    }

    /// Valideer dat alle scalars eindig en binnen toegestane ranges liggen.
    ///
    /// # Errors
    ///
    /// - [`LightingError::InvalidInstalledPower`] als `P_n < 0` of niet-eindig.
    /// - [`LightingError::InvalidFactor`] voor elke factor buiten `[0, 1]`.
    pub fn validate(&self) -> LightingCalcResult<()> {
        if !self.installed_power_w_per_m2.is_finite() || self.installed_power_w_per_m2 < 0.0 {
            return Err(LightingError::InvalidInstalledPower {
                value: self.installed_power_w_per_m2,
            });
        }
        check_factor("F_u", self.utilization_factor)?;
        check_factor("F_d", self.daylight_factor)?;
        check_factor("F_c", self.control_factor)?;
        Ok(())
    }
}

fn check_factor(name: &'static str, value: f64) -> LightingCalcResult<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        Err(LightingError::InvalidFactor { name, value })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn kantoor_forfaitair_values() {
        // Tabel 14.3: P_n;spec = 16 W/m², tabel 14.1: t_D = 2200, t_N = 300
        let s = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        assert_relative_eq!(s.installed_power_w_per_m2, 16.0, epsilon = 1e-9);
        assert_relative_eq!(s.utilization_factor, 2500.0 / 8760.0, epsilon = 1e-9);
        assert_relative_eq!(s.daylight_factor, 1.0, epsilon = 1e-9);
        assert_relative_eq!(s.control_factor, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn winkel_forfaitair_has_highest_p_n() {
        // Tabel 14.3: winkel 30 W/m² is de hoogste; grotere verlichtings-
        // intensiteit dan kantoor/school.
        let winkel = LightingSystem::forfaitair(UsageFunction::Winkelfunctie);
        let kantoor = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let school = LightingSystem::forfaitair(UsageFunction::Onderwijsfunctie);
        assert!(winkel.installed_power_w_per_m2 > kantoor.installed_power_w_per_m2);
        assert!(winkel.installed_power_w_per_m2 > school.installed_power_w_per_m2);
        assert_relative_eq!(winkel.installed_power_w_per_m2, 30.0, epsilon = 1e-9);
    }

    #[test]
    fn logies_forfaitair_higher_brandduur_than_kantoor() {
        // Hotel (logies) draait t_D = 4000, t_N = 1000 — veel hoger dan
        // kantoor (2200 / 300). F_u moet significant groter zijn.
        let logies = LightingSystem::forfaitair(UsageFunction::Logiesfunctie);
        let kantoor = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        assert!(logies.utilization_factor > kantoor.utilization_factor);
        assert_relative_eq!(logies.utilization_factor, 5000.0 / 8760.0, epsilon = 1e-9);
    }

    #[test]
    fn alle_forfaitair_waarden_geldig() {
        for usage in [
            UsageFunction::Woonfunctie,
            UsageFunction::Bijeenkomstfunctie,
            UsageFunction::Celfunctie,
            UsageFunction::Gezondheidszorgfunctie,
            UsageFunction::Industriefunctie,
            UsageFunction::Kantoorfunctie,
            UsageFunction::Logiesfunctie,
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Sportfunctie,
            UsageFunction::Winkelfunctie,
            UsageFunction::OverigeGebruiksfunctie,
        ] {
            let s = LightingSystem::forfaitair(usage);
            s.validate().unwrap_or_else(|e| {
                panic!("{usage:?}: validatie faalde: {e}");
            });
            // Plausible range voor P_n;spec in utilitaire bouw: 0-35 W/m²
            // (woonfunctie richtwaarde < 1 W/m², winkel 30 W/m² is piek).
            assert!(
                s.installed_power_w_per_m2 >= 0.0 && s.installed_power_w_per_m2 <= 35.0,
                "{usage:?}: P_n = {} buiten [0, 35]",
                s.installed_power_w_per_m2
            );
        }
    }

    #[test]
    fn new_accepts_valid() {
        let s = LightingSystem::new(10.0, 0.3, 0.7, 1.0).unwrap();
        assert_relative_eq!(s.installed_power_w_per_m2, 10.0, epsilon = 1e-12);
    }

    #[test]
    fn new_rejects_negative_power() {
        let e = LightingSystem::new(-1.0, 0.3, 1.0, 1.0).unwrap_err();
        assert!(matches!(e, LightingError::InvalidInstalledPower { .. }));
    }

    #[test]
    fn new_rejects_factor_above_one() {
        let e = LightingSystem::new(10.0, 1.5, 1.0, 1.0).unwrap_err();
        assert!(matches!(
            e,
            LightingError::InvalidFactor { name: "F_u", .. }
        ));
    }

    #[test]
    fn new_rejects_negative_factor() {
        let e = LightingSystem::new(10.0, 0.3, -0.1, 1.0).unwrap_err();
        assert!(matches!(
            e,
            LightingError::InvalidFactor { name: "F_d", .. }
        ));
    }

    #[test]
    fn new_rejects_nan_control() {
        let e = LightingSystem::new(10.0, 0.3, 1.0, f64::NAN).unwrap_err();
        assert!(matches!(
            e,
            LightingError::InvalidFactor { name: "F_c", .. }
        ));
    }

    #[test]
    fn serde_round_trip_lighting_system() {
        let s = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let json = serde_json::to_string(&s).unwrap();
        let back: LightingSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_energy_carrier_snake_case() {
        let json = serde_json::to_string(&EnergyCarrier::Electricity).unwrap();
        assert_eq!(json, "\"electricity\"");
    }
}
