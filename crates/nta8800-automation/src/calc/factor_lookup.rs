//! Lookup-tabellen voor BAC-correctiefactoren per gebruiksfunctie.

use nta8800_model::zoning::UsageFunction;

use crate::{
    errors::AutomationError,
    model::{AutomationConfig, BacsClass},
    references::{NTA_8800_2025_TABEL15_1, NTA_8800_2025_TABEL15_2},
    result::AutomationFactors,
};

/// Berekent correctiefactoren voor gebouwautomatisering per energiedienst.
///
/// Implementeert NTA 8800 H.15 tabel-lookup voor BAC-correctiefactoren
/// afhankelijk van gebruiksfunctie en automatiseringsniveau per dienst.
///
/// # Argumenten
///
/// * `config` - BAC-klassen per energiedienst
/// * `usage_function` - Gebruiksfunctie van het gebouw (woon vs. utiliteit)
///
/// # Errors
///
/// Geeft een fout als:
/// - Een energiedienst een onbekende identifier heeft
/// - De berekende factoren buiten realistische ranges (0.5-2.0) vallen
///
/// # Voorbeeld
///
/// ```
/// use nta8800_automation::{
///     calculate_automation_factors, AutomationConfig, BacsClass,
/// };
/// use nta8800_model::zoning::UsageFunction;
///
/// let config = AutomationConfig::uniform(BacsClass::B);
/// let factors = calculate_automation_factors(&config, UsageFunction::Kantoorfunctie).unwrap();
/// assert!(factors.f_bac_heating <= 1.0); // Klasse B bespaart energie
/// ```
pub fn calculate_automation_factors(
    config: &AutomationConfig,
    usage_function: UsageFunction,
) -> Result<AutomationFactors, AutomationError> {
    // Bepaal of dit een woonfunctie is of utiliteit
    let is_residential = matches!(usage_function, UsageFunction::Woonfunctie);

    let lookup_table = if is_residential {
        get_residential_factors_table()
    } else {
        get_non_residential_factors_table()
    };

    let f_bac_heating = lookup_table.get_factor(config.heating, "heating")?;
    let f_bac_cooling = lookup_table.get_factor(config.cooling, "cooling")?;
    let f_bac_lighting = lookup_table.get_factor(config.lighting, "lighting")?;
    let f_bac_dhw = lookup_table.get_factor(config.dhw, "dhw")?;
    let f_bac_ventilation = lookup_table.get_factor(config.ventilation, "ventilation")?;

    let factors = AutomationFactors::new(
        f_bac_heating,
        f_bac_cooling,
        f_bac_lighting,
        f_bac_dhw,
        f_bac_ventilation,
    );

    // Valideer dat factoren realistisch zijn
    if !factors.is_physically_realistic() {
        return Err(AutomationError::UnrealisticCorrectionFactor {
            factor: factors.average_factor(),
            service: "gemiddeld".to_string(),
        });
    }

    Ok(factors)
}

/// Lookup-tabel voor BAC-factoren.
struct BacFactorTable {
    /// Tabel-naam voor referentie.
    #[allow(dead_code)]
    table_ref: &'static str,
    /// Factoren per [BAC-klasse][energiedienst].
    factors: [[f64; 5]; 4], // [A,B,C,D] x [heating,cooling,lighting,dhw,ventilation]
}

impl BacFactorTable {
    fn get_factor(&self, bac_class: BacsClass, service: &str) -> Result<f64, AutomationError> {
        let class_idx = match bac_class {
            BacsClass::A => 0,
            BacsClass::B => 1,
            BacsClass::C => 2,
            BacsClass::D => 3,
        };

        let service_idx = match service {
            "heating" => 0,
            "cooling" => 1,
            "lighting" => 2,
            "dhw" => 3,
            "ventilation" => 4,
            _ => {
                return Err(AutomationError::MissingServiceConfiguration {
                    service: service.to_string(),
                });
            }
        };

        let factor = self.factors[class_idx][service_idx];

        // Valideer factor per dienst
        if !(0.5..=2.0).contains(&factor) {
            return Err(AutomationError::UnrealisticCorrectionFactor {
                factor,
                service: service.to_string(),
            });
        }

        Ok(factor)
    }
}

/// Geeft lookup-tabel voor woonfuncties (NTA 8800 tabel 15.1).
///
/// V1-implementatie met representatieve waarden gebaseerd op NEN-EN 15232
/// principes. Referentie: [`NTA_8800_2025_TABEL15_1`].
fn get_residential_factors_table() -> BacFactorTable {
    BacFactorTable {
        table_ref: NTA_8800_2025_TABEL15_1,
        // [A,B,C,D] x [heating,cooling,lighting,dhw,ventilation]
        factors: [
            [0.84, 0.80, 0.70, 0.91, 0.88], // Klasse A - high performance
            [0.91, 0.88, 0.84, 0.95, 0.93], // Klasse B - advanced
            [1.00, 1.00, 1.00, 1.00, 1.00], // Klasse C - standard (referentie)
            [1.20, 1.25, 1.35, 1.10, 1.15], // Klasse D - non-efficient
        ],
    }
}

/// Geeft lookup-tabel voor utiliteitsgebouwen (NTA 8800 tabel 15.2).
///
/// V1-implementatie met representatieve waarden. Utiliteitsgebouwen hebben
/// over het algemeen meer potentieel voor automatisering-besparingen door
/// complexere gebruikspatronen. Referentie: [`NTA_8800_2025_TABEL15_2`].
fn get_non_residential_factors_table() -> BacFactorTable {
    BacFactorTable {
        table_ref: NTA_8800_2025_TABEL15_2,
        // [A,B,C,D] x [heating,cooling,lighting,dhw,ventilation]
        factors: [
            [0.82, 0.77, 0.65, 0.89, 0.85], // Klasse A - hogere besparingen mogelijk
            [0.89, 0.85, 0.82, 0.93, 0.91], // Klasse B - geavanceerde regeling
            [1.00, 1.00, 1.00, 1.00, 1.00], // Klasse C - standard (referentie)
            [1.25, 1.30, 1.45, 1.12, 1.18], // Klasse D - meer inefficiëntie
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn residential_standard_bac() {
        let config = AutomationConfig::standard(); // Klasse C
        let factors = calculate_automation_factors(&config, UsageFunction::Woonfunctie).unwrap();

        // Klasse C moet 1.0 zijn (referentie)
        assert_relative_eq!(factors.f_bac_heating, 1.0);
        assert_relative_eq!(factors.f_bac_cooling, 1.0);
        assert_relative_eq!(factors.f_bac_lighting, 1.0);
        assert_relative_eq!(factors.f_bac_dhw, 1.0);
        assert_relative_eq!(factors.f_bac_ventilation, 1.0);
    }

    #[test]
    fn residential_high_performance() {
        let config = AutomationConfig::high_performance(); // Klasse A
        let factors = calculate_automation_factors(&config, UsageFunction::Woonfunctie).unwrap();

        // Klasse A moet energie besparen (< 1.0)
        assert!(factors.f_bac_heating < 1.0);
        assert!(factors.f_bac_cooling < 1.0);
        assert!(factors.f_bac_lighting < 1.0);
        assert!(factors.f_bac_dhw < 1.0);
        assert!(factors.f_bac_ventilation < 1.0);
    }

    #[test]
    fn residential_non_efficient() {
        let config = AutomationConfig::non_efficient(); // Klasse D
        let factors = calculate_automation_factors(&config, UsageFunction::Woonfunctie).unwrap();

        // Klasse D moet energie verspillen (> 1.0)
        assert!(factors.f_bac_heating > 1.0);
        assert!(factors.f_bac_cooling > 1.0);
        assert!(factors.f_bac_lighting > 1.0);
        assert!(factors.f_bac_dhw > 1.0);
        assert!(factors.f_bac_ventilation > 1.0);
    }

    #[test]
    fn non_residential_vs_residential() {
        let config = AutomationConfig::high_performance();

        let residential = calculate_automation_factors(&config, UsageFunction::Woonfunctie).unwrap();
        let office = calculate_automation_factors(&config, UsageFunction::Kantoorfunctie).unwrap();

        // Non-residential heeft over het algemeen meer besparingspotentieel
        assert!(office.f_bac_lighting <= residential.f_bac_lighting);
    }

    #[test]
    fn mixed_bac_classes() {
        let config = AutomationConfig {
            heating: BacsClass::A,    // Beste
            cooling: BacsClass::B,    // Goed
            lighting: BacsClass::C,   // Standaard
            dhw: BacsClass::D,        // Slecht
            ventilation: BacsClass::B, // Goed
        };

        let factors = calculate_automation_factors(&config, UsageFunction::Kantoorfunctie).unwrap();

        assert!(factors.f_bac_heating < 1.0);   // A = besparing
        assert!(factors.f_bac_cooling < 1.0);   // B = besparing
        assert_relative_eq!(factors.f_bac_lighting, 1.0); // C = referentie
        assert!(factors.f_bac_dhw > 1.0);        // D = verspilling
        assert!(factors.f_bac_ventilation < 1.0); // B = besparing
    }

    #[test]
    fn all_usage_functions_supported() {
        let config = AutomationConfig::standard();

        // Test alle gebruiksfuncties
        let functions = [
            UsageFunction::Woonfunctie,
            UsageFunction::Kantoorfunctie,
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Gezondheidszorgfunctie,
            UsageFunction::Winkelfunctie,
            UsageFunction::Logiesfunctie,
            UsageFunction::Bijeenkomstfunctie,
            UsageFunction::Industriefunctie,
            UsageFunction::Sportfunctie,
            UsageFunction::Celfunctie,
            UsageFunction::OverigeGebruiksfunctie,
        ];

        for function in functions {
            let result = calculate_automation_factors(&config, function);
            assert!(result.is_ok(), "Functie {function:?} niet ondersteund");
        }
    }

    #[test]
    fn factors_within_realistic_bounds() {
        let extreme_config = AutomationConfig {
            heating: BacsClass::A,
            cooling: BacsClass::A,
            lighting: BacsClass::A,
            dhw: BacsClass::D,
            ventilation: BacsClass::D,
        };

        let factors = calculate_automation_factors(&extreme_config, UsageFunction::Kantoorfunctie).unwrap();
        assert!(factors.is_physically_realistic());
    }
}