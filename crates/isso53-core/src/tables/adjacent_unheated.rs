//! Correctiefactor f_k voor warmteverlies via onverwarmde aangrenzende ruimten.
//!
//! Bron: ISSO 53 (2016) tabel 4.2, PDF p.41-42.
//!
//! f_k corrigeert het temperatuurverschil tussen ontwerpbinnentemperatuur en
//! ontwerpbuitentemperatuur voor onverwarmde ruimten met onbekende
//! binnentemperatuur. Gebruikt in formule 4.13: `H_T,iae = Σ(A_k·U_k·f_k)`.
//!
//! Voetnoten tabel 4.2 (PDF p.42):
//! 1. Een ruimte geldt als kelder wanneer ≥ 70% van de externe
//!    scheidingsconstructie onder het maaiveld ligt.
//! 2. Kruipruimte-ventilatie: zwak = openingen ≤ 1000 mm²/m²,
//!    matig = > 1000 en ≤ 1500 mm²/m², sterk = > 1500 mm²/m².

use crate::model::enums::OnverwarmdeRuimte;

/// Correctiefactor f_k voor een onverwarmde aangrenzende ruimte.
/// ISSO 53 tabel 4.2 (PDF p.41-42), dimensieloos.
pub fn f_k(ruimte: OnverwarmdeRuimte) -> f64 {
    match ruimte {
        OnverwarmdeRuimte::VertrekEenExtern => 0.4,
        OnverwarmdeRuimte::VertrekTweeExternZonderDeur => 0.5,
        OnverwarmdeRuimte::VertrekTweeExternMetDeur => 0.6,
        OnverwarmdeRuimte::VertrekDrieOfMeerExtern => 0.8,
        OnverwarmdeRuimte::KelderZonderRamenDeuren => 0.5,
        OnverwarmdeRuimte::KelderMetRamenDeuren => 0.8,
        OnverwarmdeRuimte::RuimteOnderDakHoogInfiltratie => 1.0,
        OnverwarmdeRuimte::RuimteOnderDakNietGeisoleerd => 0.9,
        OnverwarmdeRuimte::RuimteOnderDakGeisoleerd => 0.7,
        OnverwarmdeRuimte::VerkeersruimteInternLaagVentilatie => 0.0,
        OnverwarmdeRuimte::VerkeersruimteVrijGeventileerd => 1.0,
        OnverwarmdeRuimte::VerkeersruimteOverig => 0.5,
        OnverwarmdeRuimte::VloerBovenKruipruimteZwak => 0.6,
        OnverwarmdeRuimte::VloerBovenKruipruimteMatig => 0.8,
        OnverwarmdeRuimte::VloerBovenKruipruimteSterk => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertrek_externe_scheidingen() {
        assert_eq!(f_k(OnverwarmdeRuimte::VertrekEenExtern), 0.4);
        assert_eq!(f_k(OnverwarmdeRuimte::VertrekTweeExternZonderDeur), 0.5);
        assert_eq!(f_k(OnverwarmdeRuimte::VertrekTweeExternMetDeur), 0.6);
        assert_eq!(f_k(OnverwarmdeRuimte::VertrekDrieOfMeerExtern), 0.8);
    }

    #[test]
    fn test_kelder() {
        assert_eq!(f_k(OnverwarmdeRuimte::KelderZonderRamenDeuren), 0.5);
        assert_eq!(f_k(OnverwarmdeRuimte::KelderMetRamenDeuren), 0.8);
    }

    #[test]
    fn test_dak() {
        assert_eq!(f_k(OnverwarmdeRuimte::RuimteOnderDakHoogInfiltratie), 1.0);
        assert_eq!(f_k(OnverwarmdeRuimte::RuimteOnderDakNietGeisoleerd), 0.9);
        assert_eq!(f_k(OnverwarmdeRuimte::RuimteOnderDakGeisoleerd), 0.7);
    }

    #[test]
    fn test_verkeersruimte() {
        assert_eq!(f_k(OnverwarmdeRuimte::VerkeersruimteInternLaagVentilatie), 0.0);
        assert_eq!(f_k(OnverwarmdeRuimte::VerkeersruimteVrijGeventileerd), 1.0);
        assert_eq!(f_k(OnverwarmdeRuimte::VerkeersruimteOverig), 0.5);
    }

    #[test]
    fn test_kruipruimte() {
        assert_eq!(f_k(OnverwarmdeRuimte::VloerBovenKruipruimteZwak), 0.6);
        assert_eq!(f_k(OnverwarmdeRuimte::VloerBovenKruipruimteMatig), 0.8);
        assert_eq!(f_k(OnverwarmdeRuimte::VloerBovenKruipruimteSterk), 1.0);
    }
}
