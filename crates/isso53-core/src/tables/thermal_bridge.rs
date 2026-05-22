//! Toeslag voor thermische bruggen ΔU_TB.
//!
//! Bron: ISSO 53 (2016) tabel 3.1, PDF p.28.
//!
//! ΔU_TB wordt opgeteld bij de U-waarde van uitwendige scheidingsconstructies:
//! `U_eff = U + ΔU_TB` (formules 3.3 / 4.3).

/// Situatie waarvoor de thermische-bruggen-toeslag ΔU_TB geldt.
/// ISSO 53 tabel 3.1 (PDF p.28).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalBridgeSituation {
    /// Toeslagen reeds verrekend in de U-waarde.
    AlreadyInUValue,
    /// Nieuw gebouw, goede isolatie + speciale bouwkundige voorzieningen
    /// om thermische bruggen te beperken/voorkomen.
    NewWithSpecialMeasures,
    /// Nieuw gebouw gebouwd volgens de regels voor goed vakmanschap.
    NewGoodWorkmanship,
    /// Gebouwen met binnenisolatie doorbroken door plafonds.
    InteriorInsulationBrokenByCeilings,
    /// Overige situaties (default).
    Other,
}

/// Default thermische-bruggen-toeslag voor "overige situaties".
/// ISSO 53 tabel 3.1 (PDF p.28). Tevens de waarde voor de tijdconstante-
/// bepaling op gebouwniveau (§2.6.1, ΔU_TB = 0,1).
pub const DELTA_U_TB_DEFAULT: f64 = 0.10;

/// Toegevoegde warmtedoorgangscoëfficiënt ΔU_TB in W/(m²·K).
/// ISSO 53 tabel 3.1 (PDF p.28).
pub fn delta_u_tb(situation: ThermalBridgeSituation) -> f64 {
    match situation {
        ThermalBridgeSituation::AlreadyInUValue => 0.0,
        ThermalBridgeSituation::NewWithSpecialMeasures => 0.02,
        ThermalBridgeSituation::NewGoodWorkmanship => 0.05,
        ThermalBridgeSituation::InteriorInsulationBrokenByCeilings => 0.15,
        ThermalBridgeSituation::Other => DELTA_U_TB_DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_situations() {
        assert_eq!(delta_u_tb(ThermalBridgeSituation::AlreadyInUValue), 0.0);
        assert_eq!(
            delta_u_tb(ThermalBridgeSituation::NewWithSpecialMeasures),
            0.02
        );
        assert_eq!(delta_u_tb(ThermalBridgeSituation::NewGoodWorkmanship), 0.05);
        assert_eq!(
            delta_u_tb(ThermalBridgeSituation::InteriorInsulationBrokenByCeilings),
            0.15
        );
        assert_eq!(delta_u_tb(ThermalBridgeSituation::Other), 0.10);
    }

    #[test]
    fn test_default_constant() {
        assert_eq!(DELTA_U_TB_DEFAULT, 0.10);
        assert_eq!(delta_u_tb(ThermalBridgeSituation::Other), DELTA_U_TB_DEFAULT);
    }
}
