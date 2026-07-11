//! Beweegbare zonwering — effectieve g-reductie per maand.
//!
//! NTA 8800 §7.6.6.1.4, formules (7.42)/(7.43):
//!
//! ```text
//! g_gl;wi;mi = (1 − f_sh;with;mi) · g_gl;wi + f_sh;with;mi · g_gl;sh;wi      (7.42)
//! g_gl;sh;wi = F_c · g_gl;wi                                                (7.43)
//! ```
//!
//! zodat de effectieve reductiefactor op de zontoetreding per maand gelijk is
//! aan:
//!
//! ```text
//! r_mi = (1 − f_sh;with;mi) + f_sh;with;mi · F_c
//! ```
//!
//! `f_sh;with;mi` is de gewogen fractie van de tijd waarin de zonwering in
//! gebruik is en volgt uit tabel 7.7 (handbediend, woningbouw, schakelcriterium
//! 300 W/m², p. 200) of tabel 7.9 (automatisch geregeld, schakelcriterium
//! 150 W/m², p. 201). `F_c` (0..=1) is de forfaitaire reductiefactor uit tabel
//! 7.5/7.6 en wordt door de caller op [`nta8800_model::MovableSunShading`]
//! aangeleverd.
//!
//! **Bekende V1-benadering (restpunt F3d-2):** de norm schrijft voor de
//! *warmtebehoefte* van woningen `f_sh;with = 0` voor (correct ingeregelde)
//! zonwering (§7.6.6.1.4 lid 1). De demand-keten voert één gedeeld `Q_sol`-
//! profiel dat zowel de warmte- als koudebalans voedt, dus dat onderscheid kan
//! hier nog niet worden gemaakt. De tabelwaarden zijn echter ~0 in de
//! wintermaanden (nov–feb), zodat de symmetrische toepassing de warmtewinst in
//! de winter nauwelijks raakt en het reductie-effect vrijwel volledig in het
//! koelseizoen landt (fysisch de juiste vorm).

use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::ShadingControl;

/// Grens waaronder een helling als "horizontaal" wordt geklasseerd voor de
/// `f_sh;with`-kolomkeuze (schuin ≤ 22,5° → horizontaal, ≤ 67,5° → 45°,
/// daarboven verticaal). Vereenvoudiging t.o.v. de norm-interpolatie tussen
/// hellingshoeken (restpunt F3d-2).
const TILT_HORIZONTAL_MAX_DEG: f64 = 22.5;
const TILT_SLOPED_MAX_DEG: f64 = 67.5;

/// Kolom-index (0..=7) van een oriëntatie in de tabellen 7.7/7.9: volgorde
/// N, NO, O, ZO, Z, ZW, W, NW. Horizontaal heeft geen azimuth en valt op de
/// horizontale kolom.
fn orientation_index(orientation: Orientation) -> Option<usize> {
    match orientation {
        Orientation::Noord => Some(0),
        Orientation::NoordOost => Some(1),
        Orientation::Oost => Some(2),
        Orientation::ZuidOost => Some(3),
        Orientation::Zuid => Some(4),
        Orientation::ZuidWest => Some(5),
        Orientation::West => Some(6),
        Orientation::NoordWest => Some(7),
        Orientation::Horizontaal => None,
    }
}

/// Tabelblok (12 maanden × [N,NO,O,ZO,Z,ZW,W,NW]) voor de verticale (90°) en
/// de 45°-kolommen, plus de horizontale kolom (12 waarden).
struct FshWithTable {
    vertical: [[f64; 8]; 12],
    sloped_45: [[f64; 8]; 12],
    horizontal: [f64; 12],
}

/// Tabel 7.7 — handbediende zonwering, woningbouw (schakelcriterium 300 W/m²),
/// NTA 8800:2025+C1 p. 200. Verticaal 90°, schuin naar boven 45°, horizontaal
/// 0°. (De 180°-kolom — schuin naar beneden gekeerd — is 0,00 en wordt hier
/// niet apart gemodelleerd; ramen wijzen niet omlaag.)
const FSH_WITH_MANUAL_RESIDENTIAL: FshWithTable = FshWithTable {
    // rij = maand (jan..dec), kolom = N,NO,O,ZO,Z,ZW,W,NW
    vertical: [
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // jan
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // feb
        [0.00, 0.12, 0.47, 0.64, 0.68, 0.55, 0.31, 0.00], // mrt
        [0.00, 0.29, 0.59, 0.70, 0.71, 0.66, 0.48, 0.13], // apr
        [0.00, 0.30, 0.56, 0.65, 0.67, 0.60, 0.50, 0.23], // mei
        [0.00, 0.32, 0.51, 0.52, 0.56, 0.56, 0.57, 0.35], // jun
        [0.00, 0.25, 0.49, 0.55, 0.59, 0.54, 0.51, 0.30], // jul
        [0.00, 0.08, 0.44, 0.63, 0.68, 0.70, 0.61, 0.28], // aug
        [0.00, 0.01, 0.43, 0.66, 0.70, 0.66, 0.48, 0.07], // sep
        [0.00, 0.00, 0.39, 0.67, 0.69, 0.62, 0.33, 0.00], // okt
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // nov
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // dec
    ],
    sloped_45: [
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // jan
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // feb
        [0.00, 0.20, 0.56, 0.70, 0.73, 0.68, 0.47, 0.10], // mrt
        [0.00, 0.49, 0.71, 0.78, 0.80, 0.76, 0.67, 0.43], // apr
        [0.16, 0.59, 0.75, 0.79, 0.82, 0.79, 0.73, 0.58], // mei
        [0.54, 0.58, 0.71, 0.74, 0.75, 0.77, 0.71, 0.64], // jun
        [0.34, 0.55, 0.66, 0.74, 0.74, 0.75, 0.68, 0.57], // jul
        [0.00, 0.44, 0.67, 0.79, 0.82, 0.82, 0.75, 0.57], // aug
        [0.00, 0.19, 0.60, 0.74, 0.76, 0.72, 0.62, 0.25], // sep
        [0.00, 0.01, 0.50, 0.69, 0.71, 0.67, 0.47, 0.00], // okt
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // nov
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00], // dec
    ],
    horizontal: [
        0.00, 0.00, 0.56, 0.72, 0.79, 0.79, 0.74, 0.81, 0.65, 0.48, 0.00, 0.00,
    ],
};

/// Tabel 7.9 — automatisch geregelde zonwering, woning- én utiliteitsbouw
/// (schakelcriterium 150 W/m²), NTA 8800:2025+C1 p. 201.
const FSH_WITH_AUTOMATIC: FshWithTable = FshWithTable {
    vertical: [
        [0.00, 0.00, 0.45, 0.78, 0.86, 0.80, 0.48, 0.00], // jan
        [0.00, 0.04, 0.54, 0.74, 0.79, 0.73, 0.44, 0.03], // feb
        [0.00, 0.23, 0.65, 0.81, 0.86, 0.78, 0.55, 0.10], // mrt
        [0.12, 0.51, 0.75, 0.83, 0.88, 0.81, 0.71, 0.46], // apr
        [0.25, 0.59, 0.75, 0.81, 0.85, 0.81, 0.73, 0.58], // mei
        [0.36, 0.63, 0.73, 0.75, 0.79, 0.79, 0.78, 0.68], // jun
        [0.34, 0.59, 0.72, 0.75, 0.78, 0.76, 0.73, 0.63], // jul
        [0.22, 0.46, 0.70, 0.81, 0.88, 0.86, 0.79, 0.62], // aug
        [0.04, 0.21, 0.64, 0.84, 0.88, 0.84, 0.68, 0.30], // sep
        [0.00, 0.05, 0.59, 0.83, 0.87, 0.82, 0.51, 0.03], // okt
        [0.00, 0.00, 0.50, 0.79, 0.82, 0.78, 0.37, 0.00], // nov
        [0.00, 0.00, 0.41, 0.79, 0.86, 0.79, 0.43, 0.00], // dec
    ],
    sloped_45: [
        [0.00, 0.00, 0.51, 0.77, 0.81, 0.78, 0.54, 0.00], // jan
        [0.01, 0.19, 0.61, 0.77, 0.79, 0.75, 0.58, 0.12], // feb
        [0.29, 0.62, 0.80, 0.88, 0.89, 0.86, 0.76, 0.57], // mrt
        [0.56, 0.78, 0.87, 0.92, 0.92, 0.91, 0.86, 0.78], // apr
        [0.86, 0.83, 0.88, 0.91, 0.92, 0.91, 0.89, 0.83], // mei
        [0.90, 0.86, 0.88, 0.90, 0.91, 0.91, 0.89, 0.88], // jun
        [0.84, 0.82, 0.86, 0.88, 0.89, 0.89, 0.86, 0.83], // jul
        [0.84, 0.80, 0.87, 0.91, 0.94, 0.92, 0.91, 0.86], // aug
        [0.36, 0.66, 0.81, 0.89, 0.91, 0.90, 0.84, 0.70], // sep
        [0.05, 0.32, 0.74, 0.86, 0.88, 0.85, 0.70, 0.32], // okt
        [0.00, 0.00, 0.56, 0.76, 0.81, 0.76, 0.55, 0.00], // nov
        [0.00, 0.00, 0.51, 0.76, 0.82, 0.75, 0.48, 0.00], // dec
    ],
    horizontal: [
        0.48, 0.67, 0.84, 0.91, 0.92, 0.92, 0.92, 0.93, 0.89, 0.79, 0.58, 0.42,
    ],
};

fn table_for(control: ShadingControl) -> &'static FshWithTable {
    match control {
        ShadingControl::ManualResidential => &FSH_WITH_MANUAL_RESIDENTIAL,
        ShadingControl::Automatic => &FSH_WITH_AUTOMATIC,
    }
}

/// Gewogen inzetfractie `f_sh;with` per maand voor een raam met beweegbare
/// zonwering, NTA 8800 tabel 7.7/7.9.
///
/// De hellingskolom wordt geforfaiteerd: helling ≤ 22,5° → horizontaal,
/// ≤ 67,5° → 45°, daarboven verticaal (norm-interpolatie = restpunt F3d-2).
#[must_use]
pub fn fsh_with(orientation: Orientation, tilt: Tilt, control: ShadingControl) -> MonthlyProfile<f64> {
    let table = table_for(control);
    let mut out = [0.0_f64; 12];

    // Horizontaal (plat) raam of ontbrekende azimuth → horizontale kolom.
    if tilt.degrees <= TILT_HORIZONTAL_MAX_DEG || orientation_index(orientation).is_none() {
        return MonthlyProfile::new(table.horizontal);
    }
    let Some(col) = orientation_index(orientation) else {
        return MonthlyProfile::new(table.horizontal);
    };
    let block = if tilt.degrees <= TILT_SLOPED_MAX_DEG {
        &table.sloped_45
    } else {
        &table.vertical
    };
    for month in Month::all() {
        out[month.index()] = block[month.index()][col];
    }
    MonthlyProfile::new(out)
}

/// Effectieve g-reductiefactor per maand voor beweegbare zonwering
/// (formule 7.42/7.43): `r_mi = (1 − f_sh;with;mi) + f_sh;with;mi · F_c`.
///
/// `f_c` wordt geklemd op `0..=1`. Zonder zonwering (caller levert geen
/// [`nta8800_model::MovableSunShading`]) hoort deze functie niet te worden
/// aangeroepen; de reductie is dan per definitie 1,0.
#[must_use]
pub fn movable_shading_g_factor(
    f_c: f64,
    orientation: Orientation,
    tilt: Tilt,
    control: ShadingControl,
) -> MonthlyProfile<f64> {
    let f_c = f_c.clamp(0.0, 1.0);
    let fsh = fsh_with(orientation, tilt, control);
    let mut out = [0.0_f64; 12];
    for month in Month::all() {
        let f = fsh[month];
        out[month.index()] = (1.0 - f) + f * f_c;
    }
    MonthlyProfile::new(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winter_manual_residential_geen_inzet() {
        // Tabel 7.7: nov–feb overal 0,00 → g-factor = 1,0 (geen reductie).
        let f = movable_shading_g_factor(0.2, Orientation::Zuid, Tilt::VERTICAL, ShadingControl::ManualResidential);
        for m in [Month::Januari, Month::Februari, Month::November, Month::December] {
            assert!((f[m] - 1.0).abs() < 1e-12, "{m:?} = {}", f[m]);
        }
    }

    #[test]
    fn zomer_zuid_reduceert() {
        // Juli, Zuid vert, handbediend: f_sh;with = 0,59; F_c = 0,20.
        // r = (1−0,59) + 0,59·0,20 = 0,41 + 0,118 = 0,528.
        let f = movable_shading_g_factor(0.2, Orientation::Zuid, Tilt::VERTICAL, ShadingControl::ManualResidential);
        assert!((f[Month::Juli] - 0.528).abs() < 1e-9, "juli = {}", f[Month::Juli]);
    }

    #[test]
    fn automatic_reduceert_meer_dan_manual_in_de_zomer() {
        let manual = movable_shading_g_factor(0.2, Orientation::Zuid, Tilt::VERTICAL, ShadingControl::ManualResidential);
        let auto = movable_shading_g_factor(0.2, Orientation::Zuid, Tilt::VERTICAL, ShadingControl::Automatic);
        assert!(auto[Month::Juli] < manual[Month::Juli]);
    }

    #[test]
    fn fc_1_geeft_geen_reductie() {
        // F_c = 1 → g_gl;sh = g_gl → r = 1 in alle maanden.
        let f = movable_shading_g_factor(1.0, Orientation::ZuidWest, Tilt::VERTICAL, ShadingControl::Automatic);
        for m in Month::all() {
            assert!((f[m] - 1.0).abs() < 1e-12, "{m:?} = {}", f[m]);
        }
    }

    #[test]
    fn factor_binnen_fc_en_1() {
        let f = movable_shading_g_factor(0.15, Orientation::West, Tilt::VERTICAL, ShadingControl::Automatic);
        for m in Month::all() {
            assert!(f[m] >= 0.15 - 1e-12 && f[m] <= 1.0 + 1e-12, "{m:?} = {}", f[m]);
        }
    }

    #[test]
    fn horizontaal_gebruikt_horizontale_kolom() {
        let f = fsh_with(Orientation::Horizontaal, Tilt::HORIZONTAL, ShadingControl::Automatic);
        // Juli horizontaal automatisch = 0,92.
        assert!((f[Month::Juli] - 0.92).abs() < 1e-12, "juli = {}", f[Month::Juli]);
    }
}
