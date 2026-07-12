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
use nta8800_model::{Obstruction, ShadingControl};

/// Balans-tak waarvoor een zonwinst-reductie wordt bepaald.
///
/// De NTA 8800 houdt de warmte- en koudebalans strikt gescheiden voor beide
/// beschaduwingsmechanismen:
/// - Beweegbare zonwering: voor de **warmtebehoefte van woningen** geldt
///   `f_sh;with = 0` (§7.6.6.1.4 lid 1) — correct ingeregelde zonwering wordt
///   bij de warmtevraag niet ingezet, dus geen g-reductie. Bij de koudevraag
///   geldt het volledige maandprofiel (tabel 7.7/7.9).
/// - Externe belemmering (§17.3): tabel 17.4 voor verwarming, tabel 17.5 voor
///   koeling. Bij minimale belemmering is 17.5 uniform 1,00; 17.4 kent
///   winterreducties (de standaard-horizon blokkeert de lage winterzon).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolarBalance {
    /// Warmtebalans (`Q_H;nd`).
    Heating,
    /// Koudebalans (`Q_C;nd`).
    Cooling,
}

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

// ---------------------------------------------------------------------------
// Externe belemmering — NTA 8800 §17.3 (formule 7.33, factor F_sh;obst;mi)
// ---------------------------------------------------------------------------

/// Tabelblok voor `F_sh;obst;mi` bij minimale belemmering, zelfde indeling als
/// [`FshWithTable`]: 12 maanden × [N,NO,O,ZO,Z,ZW,W,NW] voor de verticale (90°)
/// en de 45°-kolom, plus de horizontale kolom (0°, oriëntatie-onafhankelijk).
struct ObstructionTable {
    vertical: [[f64; 8]; 12],
    sloped_45: [[f64; 8]; 12],
    horizontal: [f64; 12],
}

/// Tabel 17.4 — `F_sh;obst;mi` bij minimale belemmering (§17.3.2 onder a) voor
/// de **warmtebehoefteberekening**, NTA 8800:2025+C1 p. 708-714. Verticaal 90°,
/// schuin naar boven 45°, horizontaal 0°. De lage winterwaarden voor de
/// zuidgeoriënteerde kolommen weerspiegelen de standaard-horizon die de lage
/// winterzon blokkeert; de horizontale kolom is per oriëntatie identiek 1,00.
/// (Transcriptie via PyMuPDF, pixmap-geverifieerd — zie het F3d-analyse-doc.)
const FSH_OBST_HEATING: ObstructionTable = ObstructionTable {
    // rij = maand (jan..dec), kolom = N,NO,O,ZO,Z,ZW,W,NW
    vertical: [
        [1.00, 1.00, 0.92, 0.48, 0.23, 0.49, 0.85, 0.97], // jan
        [1.00, 0.96, 0.79, 0.81, 0.91, 0.83, 0.85, 0.97], // feb
        [1.00, 0.97, 0.82, 0.87, 1.00, 0.93, 0.89, 0.96], // mrt
        [0.99, 0.97, 0.91, 0.95, 1.00, 0.92, 0.82, 0.87], // apr
        [0.97, 0.93, 0.95, 1.00, 1.00, 0.99, 0.88, 0.85], // mei
        [0.97, 0.88, 0.90, 1.00, 1.00, 1.00, 0.93, 0.91], // jun
        [0.97, 0.91, 0.93, 0.99, 1.00, 1.00, 0.92, 0.90], // jul
        [0.98, 0.98, 0.94, 0.98, 1.00, 0.99, 0.89, 0.88], // aug
        [1.00, 0.97, 0.87, 0.92, 1.00, 0.91, 0.85, 0.96], // sep
        [1.00, 0.96, 0.84, 0.86, 0.97, 0.88, 0.83, 0.97], // okt
        [1.00, 0.98, 0.92, 0.70, 0.61, 0.71, 0.90, 0.99], // nov
        [1.00, 1.00, 0.86, 0.40, 0.19, 0.58, 0.87, 1.00], // dec
    ],
    sloped_45: [
        [1.00, 0.99, 0.92, 0.56, 0.29, 0.57, 0.86, 0.97], // jan
        [1.00, 0.92, 0.83, 0.86, 0.93, 0.87, 0.85, 0.93], // feb
        [1.00, 0.88, 0.83, 0.89, 1.00, 0.94, 0.86, 0.89], // mrt
        [0.84, 0.80, 0.88, 0.93, 1.00, 0.93, 0.79, 0.79], // apr
        [0.65, 0.74, 0.89, 0.96, 0.99, 0.95, 0.80, 0.70], // mei
        [0.68, 0.78, 0.85, 0.96, 0.96, 0.94, 0.86, 0.75], // jun
        [0.69, 0.81, 0.89, 0.96, 0.98, 0.93, 0.85, 0.77], // jul
        [0.74, 0.80, 0.88, 0.94, 0.99, 0.96, 0.83, 0.76], // aug
        [0.96, 0.85, 0.86, 0.92, 1.00, 0.91, 0.81, 0.87], // sep
        [1.00, 0.90, 0.81, 0.89, 0.97, 0.90, 0.86, 0.92], // okt
        [1.00, 0.95, 0.86, 0.76, 0.66, 0.77, 0.92, 0.97], // nov
        [1.00, 0.99, 0.90, 0.48, 0.27, 0.65, 0.85, 1.00], // dec
    ],
    // Tabel 17.4, kolom 0° (Hor.): per oriëntatie identiek 1,00 (geen
    // azimuth-afhankelijke horizonblokkering voor een plat vlak).
    horizontal: [1.00; 12],
};

/// `F_sh;obst;mi` per maand voor een raam, NTA 8800 §17.3.
///
/// - [`Obstruction::None`] → 1,0 in elke maand (geen belemmering).
/// - [`Obstruction::Minimal`] + [`SolarBalance::Heating`] → tabel 17.4.
/// - [`Obstruction::Minimal`] + [`SolarBalance::Cooling`] → tabel 17.5, die bij
///   minimale belemmering uniform 1,00 is ("Elke oriëntatie / Elke maand",
///   NTA 8800:2025+C1 p. 715) — dus geen koudereductie.
///
/// De hellingskolom wordt geforfaiteerd volgens dezelfde bucketing als
/// [`fsh_with`] (≤ 22,5° → horizontaal, ≤ 67,5° → 45°, daarboven verticaal;
/// norm-interpolatie tussen hellingshoeken = restpunt).
#[must_use]
pub fn obstruction_g_factor(
    obstruction: Obstruction,
    orientation: Orientation,
    tilt: Tilt,
    balance: SolarBalance,
) -> MonthlyProfile<f64> {
    // Tabel 17.5 (koeling, minimale belemmering) is uniform 1,00 → alleen de
    // warmtebalans kent een belemmeringsreductie.
    if obstruction == Obstruction::None || balance == SolarBalance::Cooling {
        return MonthlyProfile::from_constant(1.0);
    }
    let table = &FSH_OBST_HEATING;

    // Horizontaal (plat) raam of ontbrekende azimuth → horizontale kolom (1,00).
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
    let mut out = [0.0_f64; 12];
    for month in Month::all() {
        out[month.index()] = block[month.index()][col];
    }
    MonthlyProfile::new(out)
}

// ---------------------------------------------------------------------------
// F_c-forfaits — NTA 8800 tabel 7.5/7.6 (p. 199)
// ---------------------------------------------------------------------------

/// Type beweegbare zonwering met een oriëntatie-onafhankelijke `F_c`-forfait
/// uit NTA 8800 tabel 7.5 (p. 199). De kleur-varianten dekken het
/// schakelcriterium (`T_s`/`R_s`); "onbekend" is de altijd-toepasbare default.
///
/// De caller mag `F_c` ook rechtstreeks op [`nta8800_model::MovableSunShading`]
/// zetten; deze enum is een norm-verankerde afleiding voor het gangbare geval.
/// Oriëntatie-afhankelijke uitval-/knikarmschermen (tabel 7.6) staan in
/// [`awning_f_c`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SunShadingType {
    /// Buitenscreen, zwart/antraciet/donkerbruin (`T_s < 0,07`).
    ScreenDark,
    /// Buitenscreen, overige kleuren (`T_s < 0,17`).
    ScreenColored,
    /// Buitenscreen, wit (`T_s ≥ 0,17`).
    ScreenWhite,
    /// Buitenscreen, onbekende kleur (default).
    ScreenUnknown,
    /// Buitenjaloezie, zwart/antraciet/donkerbruin (`R_s < 0,3`).
    BlindDark,
    /// Buitenjaloezie, overige kleuren (`R_s < 0,6`).
    BlindColored,
    /// Buitenjaloezie, wit (`R_s ≥ 0,6`).
    BlindWhite,
    /// Buitenjaloezie, onbekende kleur.
    BlindUnknown,
    /// Buitenrolluik, overige kleuren (`R_s ≤ 0,70`).
    RollerShutterColored,
    /// Buitenrolluik, wit (`R_s > 0,70`).
    RollerShutterWhite,
    /// Buitenrolluik, onbekende kleur.
    RollerShutterUnknown,
    /// Gemetalliseerd weefsel, binnen toegepast (`R_s > 0,72`).
    MetallizedFabricInterior,
}

impl SunShadingType {
    /// Forfaitaire `F_c` uit NTA 8800 tabel 7.5 (p. 199).
    #[must_use]
    // Norm-onderscheiden zonwering-categorieën die toevallig hetzelfde forfait
    // delen (bv. onbekend screen 0,20 vs witte jaloezie 0,20) blijven bewust
    // aparte match-armen — de tabelstructuur is dan 1-op-1 herleidbaar.
    #[allow(clippy::match_same_arms)]
    pub fn f_c(self) -> f64 {
        match self {
            SunShadingType::ScreenDark => 0.12,
            SunShadingType::ScreenColored | SunShadingType::ScreenUnknown => 0.20,
            SunShadingType::ScreenWhite => 0.25,
            SunShadingType::BlindDark => 0.05,
            SunShadingType::BlindColored | SunShadingType::BlindUnknown => 0.10,
            SunShadingType::BlindWhite => 0.20,
            SunShadingType::RollerShutterColored | SunShadingType::RollerShutterUnknown => 0.11,
            SunShadingType::RollerShutterWhite => 0.04,
            SunShadingType::MetallizedFabricInterior => 0.45,
        }
    }
}

/// Uitval-/knikarmscherm — oriëntatie-afhankelijke `F_c` uit NTA 8800 tabel 7.6
/// (p. 199, gebaseerd op HR++-glas). Voor tussenliggende oriëntaties geldt de
/// dichtstbijzijnde; de tabel groepeert {NO,NW}, {O,W} en {ZO,ZW}. Horizontaal
/// heeft geen azimuth → valt op de Noord-kolom (conservatief hoogst).
#[must_use]
pub fn awning_f_c(retractable: bool, orientation: Orientation) -> f64 {
    // Kolommen: N | NO,NW | O,W | ZO,ZW | Z
    let (n, no_nw, o_w, zo_zw, z) = if retractable {
        // Knikarmscherm.
        (0.90, 0.80, 0.65, 0.55, 0.50)
    } else {
        // Uitvalscherm.
        (0.50, 0.45, 0.35, 0.35, 0.35)
    };
    match orientation {
        Orientation::Noord | Orientation::Horizontaal => n,
        Orientation::NoordOost | Orientation::NoordWest => no_nw,
        Orientation::Oost | Orientation::West => o_w,
        Orientation::ZuidOost | Orientation::ZuidWest => zo_zw,
        Orientation::Zuid => z,
    }
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

    // --- Externe belemmering §17.3 (tabel 17.4/17.5) ---

    #[test]
    fn obstruction_none_is_altijd_een() {
        for balance in [SolarBalance::Heating, SolarBalance::Cooling] {
            let f =
                obstruction_g_factor(Obstruction::None, Orientation::Zuid, Tilt::VERTICAL, balance);
            for m in Month::all() {
                assert!((f[m] - 1.0).abs() < 1e-12, "{m:?} = {}", f[m]);
            }
        }
    }

    #[test]
    fn obstruction_cooling_is_altijd_een() {
        // Tabel 17.5 minimale belemmering = uniform 1,00.
        let f = obstruction_g_factor(
            Obstruction::Minimal,
            Orientation::Zuid,
            Tilt::VERTICAL,
            SolarBalance::Cooling,
        );
        for m in Month::all() {
            assert!((f[m] - 1.0).abs() < 1e-12, "{m:?} = {}", f[m]);
        }
    }

    #[test]
    fn obstruction_heating_zuid_verticaal_blokkeert_winter() {
        // Tabel 17.4 Zuid vert.: jan 0,23 / dec 0,19 (lage winterzon geblokkeerd),
        // zomer ≈ 1,00.
        let f = obstruction_g_factor(
            Obstruction::Minimal,
            Orientation::Zuid,
            Tilt::VERTICAL,
            SolarBalance::Heating,
        );
        assert!((f[Month::Januari] - 0.23).abs() < 1e-12, "jan = {}", f[Month::Januari]);
        assert!((f[Month::December] - 0.19).abs() < 1e-12, "dec = {}", f[Month::December]);
        assert!((f[Month::Juni] - 1.00).abs() < 1e-12, "jun = {}", f[Month::Juni]);
        // Winter fors gereduceerd t.o.v. zomer.
        assert!(f[Month::December] < 0.5 * f[Month::Juni]);
    }

    #[test]
    fn obstruction_heating_horizontaal_is_een() {
        // Tabel 17.4 kolom 0° = 1,00 voor elke oriëntatie.
        let f = obstruction_g_factor(
            Obstruction::Minimal,
            Orientation::Horizontaal,
            Tilt::HORIZONTAL,
            SolarBalance::Heating,
        );
        for m in Month::all() {
            assert!((f[m] - 1.0).abs() < 1e-12, "{m:?} = {}", f[m]);
        }
    }

    #[test]
    fn obstruction_heating_tabel_steekproef() {
        // ≥6 H-waarden tegen de getranscribeerde tabel 17.4 (p. 708-714),
        // pixmap-geverifieerd. (orientatie, tilt, maand, verwachte F_sh;obst).
        let cases = [
            (Orientation::Zuid, Tilt::VERTICAL, Month::Januari, 0.23),
            (Orientation::Zuid, Tilt::VERTICAL, Month::November, 0.61),
            (Orientation::Oost, Tilt::VERTICAL, Month::Februari, 0.79),
            (Orientation::Oost, Tilt::VERTICAL, Month::December, 0.86),
            (Orientation::ZuidOost, Tilt::VERTICAL, Month::Januari, 0.48),
            (Orientation::West, Tilt::VERTICAL, Month::April, 0.82),
            (Orientation::Noord, Tilt::VERTICAL, Month::Mei, 0.97),
            (Orientation::Zuid, Tilt { degrees: 45.0 }, Month::Januari, 0.29),
            (Orientation::Oost, Tilt { degrees: 45.0 }, Month::Mei, 0.89),
        ];
        for (o, t, m, expected) in cases {
            let f = obstruction_g_factor(Obstruction::Minimal, o, t, SolarBalance::Heating);
            assert!(
                (f[m] - expected).abs() < 1e-12,
                "{o:?} {t:?} {m:?}: {} != {expected}",
                f[m]
            );
        }
    }

    // --- F_c-forfaits tabel 7.5/7.6 ---

    #[test]
    fn f_c_forfaits_tabel_7_5() {
        assert!((SunShadingType::ScreenDark.f_c() - 0.12).abs() < 1e-12);
        assert!((SunShadingType::ScreenUnknown.f_c() - 0.20).abs() < 1e-12);
        assert!((SunShadingType::ScreenWhite.f_c() - 0.25).abs() < 1e-12);
        assert!((SunShadingType::BlindDark.f_c() - 0.05).abs() < 1e-12);
        assert!((SunShadingType::RollerShutterWhite.f_c() - 0.04).abs() < 1e-12);
        assert!((SunShadingType::MetallizedFabricInterior.f_c() - 0.45).abs() < 1e-12);
    }

    #[test]
    fn awning_f_c_tabel_7_6() {
        // Uitvalscherm: N 0,50 / Z 0,35. Knikarmscherm: N 0,90 / Z 0,50.
        assert!((awning_f_c(false, Orientation::Noord) - 0.50).abs() < 1e-12);
        assert!((awning_f_c(false, Orientation::Zuid) - 0.35).abs() < 1e-12);
        assert!((awning_f_c(true, Orientation::Noord) - 0.90).abs() < 1e-12);
        assert!((awning_f_c(true, Orientation::Zuid) - 0.50).abs() < 1e-12);
        // Groepering {O,W}: knikarm = 0,65.
        assert!((awning_f_c(true, Orientation::Oost) - awning_f_c(true, Orientation::West)).abs() < 1e-12);
        assert!((awning_f_c(true, Orientation::Oost) - 0.65).abs() < 1e-12);
    }
}
