//! Bijlage AA NTA 8800:2025 — volledige vereenvoudigde bepalingsmethode
//! koelbehoefte + minimaal benodigde koelcapaciteit voor woningen.
//!
//! Deze module is een **complete, zelfdragende implementatie** van bijlage AA
//! op basis van het 2025-concept (publicatie internetconsultatie EPG2026,
//! versie 2025-03 / 2025-04 zoals weerspiegeld in de RVO-rekentool).
//!
//! Verschil met [`crate::simplified`]:
//!
//! - [`crate::simplified`] biedt low-level building blocks die per formule
//!   ingezet kunnen worden, maar verwacht P_sol en P_gl als caller-supplied
//!   inputs en heeft geen geïntegreerde tabel AA.3.
//! - [`bijlage_aa`](self) bevat de **volledige berekening** voor een woning
//!   inclusief tabel AA.3 (β × γ × tijdstip matrix met lineaire β-interpolatie
//!   en dichtstbijzijnde-oriëntatie regel voor γ), AA.6 (zoninstraling per
//!   verblijfsruimte met max over 9..18 h), AA.7 (glas-transmissie) en AA.8
//!   t/m AA.13 in één call.
//!
//! ## Bronnen
//!
//! 1. Concept-tekst bijlage AA (internetconsultatie EPG2026, oktober 2025) —
//!    `https://www.internetconsultatie.nl/epg2026/document/14174`
//! 2. RVO-rekentool xlsm versie 2025.04 — `tests/references/
//!    rekentool-bijlage-aa-nta8800-2025.04.xlsm` (gitignored)
//! 3. Tabel-waarden voor f_iso, F_c, mass-class en tijdstip-AA.2 zijn
//!    overgenomen uit de "Tabellen" en "Tabel AA" sheets van de xlsm.
//!
//! ## Scope V1
//!
//! - Volledige formules AA.1 t/m AA.13 voor één rekenzone met ≥1 verblijfsruimte
//! - Tabel AA.1, AA.2, AA.3 hardcoded (geen externe lookup)
//! - SWM-klasse + tijdstip-AA.2 lookup voor "tijdstip maximale koellast" als
//!   alternatief invoer-pad (naast expliciete `peak_hour` per ruimte)
//! - Lineaire interpolatie tussen β-waarden in tabel AA.3
//! - "Dichtstbijzijnde oriëntatie" voor tussenliggende γ met hoogste-bij-gelijk regel
//!
//! ## Niet in V1 (zie [`TODO`](#todos))
//!
//! - F_C (zonwering) lookup via [`ZonweringType`] enum is geïmplementeerd,
//!   maar uitvalschermen/knikarmschermen krijgen `Fc = 1.0` (norm verwijst
//!   naar tabel 7.5, deze waarden zijn niet in de rekentool gespecificeerd).
//! - F_sh (beschaduwingsreductie) wordt per raam als directe `f_sh` ingegeven;
//!   automatische afleiding uit overstek/zijbelemmering volgens tabellen 17.5,
//!   17.9 en 17.11 is uit scope (vereist gebouw-geometrie).
//! - SWM-bepaling op basis van bouwwijze (tabellen 7.11/7.12) is hardcoded
//!   maar de relatie SWM → tijdstip is door de norm zelf afgezwakt
//!   ("SWM is in deze methode geen rekenparameter, gebouw wordt geladen
//!   verondersteld" — AA.2 stap 1). Tabel AA.2 (tijdstip per oriëntatie)
//!   blijft beschikbaar als hulpfunctie maar wordt door AA.6 niet meer
//!   normatief gebruikt.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{CoolingCalcResult, CoolingError};

// ---------------------------------------------------------------------------
// Constanten (fysica + vaste waardes uit bijlage AA)
// ---------------------------------------------------------------------------

/// Dichtheid van lucht ρ_a, in kg/m³ (formule AA.4).
pub const RHO_AIR_KG_PER_M3: f64 = 1.205;

/// Specifieke warmtecapaciteit van lucht c_a, in J/(kg·K) (formule AA.4).
pub const C_AIR_J_PER_KGK: f64 = 1005.0;

/// Binnentemperatuur tijdens koelpiek θ_i = 24 °C (zie AA.4, AA.7).
pub const INDOOR_TEMPERATURE_C: f64 = 24.0;

/// Vaste aftrek in formules AA.11/AA.13, in W/m². Komt overeen met de
/// situatie waarin net voldaan wordt aan TO_juli < 1,2 of GTO < 450 h.
pub const FIXED_DEDUCTION_W_PER_M2: f64 = 35.0;

/// Gemiddelde oppervlakteverhouding raam-glasvlak (factor 0,75 in AA.6b).
pub const RAAM_GLASVLAK_RATIO: f64 = 0.75;

// ---------------------------------------------------------------------------
// Tabel AA.1 — buitenluchttemperatuur per tijdstip
// ---------------------------------------------------------------------------

/// Tabel AA.1 — θ_e [°C] per uur (9..21 h). 13 waarden uit NEN 5060:2018+A1:2021.
pub const TABEL_AA_1_THETA_E: [(u8, f64); 13] = [
    (9, 24.7),
    (10, 26.9),
    (11, 28.2),
    (12, 28.9),
    (13, 29.7),
    (14, 29.9),
    (15, 29.8),
    (16, 30.4),
    (17, 30.6),
    (18, 30.1),
    (19, 29.5),
    (20, 25.9),
    (21, 23.4),
];

/// Zoek θ_e uit tabel AA.1 op tijdstip `uur` (9..=21).
#[must_use]
pub fn theta_e(uur: u8) -> Option<f64> {
    TABEL_AA_1_THETA_E
        .iter()
        .find(|(h, _)| *h == uur)
        .map(|(_, t)| *t)
}

// ---------------------------------------------------------------------------
// Tabel AA.2 — bouwjaarklasse → f_iso
// ---------------------------------------------------------------------------

/// Bouwjaarklasse voor f_iso (tabel AA.2 in de norm, "Tabel AA.4" in de
/// xlsm — zelfde tabel, andere nummering).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum BouwjaarKlasseAa {
    /// Bouwjaar ≤ 1975 — f_iso = 17 W/m².
    Tot1975,
    /// 1975 ≤ bouwjaar < 1992 — f_iso = 10 W/m².
    Van1975Tot1992,
    /// 1992 ≤ bouwjaar < 2015 — f_iso = 3,2 W/m².
    Van1992Tot2015,
    /// bouwjaar ≥ 2015 — f_iso = 2,2 W/m².
    Van2015,
}

impl BouwjaarKlasseAa {
    /// Leid bouwjaarklasse af uit een numeriek bouwjaar.
    #[must_use]
    pub const fn from_year(year: u32) -> Self {
        if year < 1975 {
            Self::Tot1975
        } else if year < 1992 {
            Self::Van1975Tot1992
        } else if year < 2015 {
            Self::Van1992Tot2015
        } else {
            Self::Van2015
        }
    }

    /// f_iso [W/m²] uit tabel AA.2.
    #[must_use]
    pub const fn f_iso(self) -> f64 {
        match self {
            Self::Tot1975 => 17.0,
            Self::Van1975Tot1992 => 10.0,
            Self::Van1992Tot2015 => 3.2,
            Self::Van2015 => 2.2,
        }
    }
}

// ---------------------------------------------------------------------------
// Tabel 7.5 — F_c per zonwering-type
// ---------------------------------------------------------------------------

/// Type zonwering — F_c uit tabel 7.5 NTA 8800 (zoals overgenomen in de
/// RVO-rekentool 2025.04). Uitvalschermen en knikarmschermen krijgen
/// `Fc = 1.0` als conservatieve placeholder; deze waarden zijn nog niet
/// in de rekentool gespecificeerd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ZonweringType {
    /// Geen zonwering — F_c = 1,00.
    Geen,
    /// Screen (zwart, antraciet, donkerbruin) — F_c = 0,12.
    ScreenDonker,
    /// Screen (overige kleuren) — F_c = 0,20.
    ScreenOverig,
    /// Screen (wit) — F_c = 0,25.
    ScreenWit,
    /// Jaloezieën (zwart, antraciet, donkerbruin) — F_c = 0,05.
    JaloeziDonker,
    /// Jaloezieën (overige kleuren) — F_c = 0,10.
    JaloeziOverig,
    /// Jaloezieën (wit) — F_c = 0,20.
    JaloeziWit,
    /// Rolluiken (overige kleuren) — F_c = 0,11.
    RolluikOverig,
    /// Rolluiken (wit) — F_c = 0,04.
    RolluikWit,
    /// Gemetalliseerde weefsels (binnen toegepast) — F_c = 0,45.
    GemetWeefselBinnen,
    /// Uitvalscherm — placeholder F_c = 1,00 (norm verwijst naar tabel 7.5
    /// voor specifieke waarde, rekentool laat veld leeg).
    Uitvalscherm,
    /// Knikarmscherm — placeholder F_c = 1,00 (idem).
    Knikarmscherm,
}

impl ZonweringType {
    /// F_c (reductiefactor zonwering) voor dit type, uit tabel 7.5 NTA 8800.
    #[must_use]
    pub const fn f_c(self) -> f64 {
        match self {
            Self::Geen | Self::Uitvalscherm | Self::Knikarmscherm => 1.0,
            Self::ScreenDonker => 0.12,
            Self::ScreenOverig => 0.20,
            Self::ScreenWit => 0.25,
            Self::JaloeziDonker => 0.05,
            Self::JaloeziOverig => 0.10,
            Self::JaloeziWit => 0.20,
            Self::RolluikOverig => 0.11,
            Self::RolluikWit => 0.04,
            Self::GemetWeefselBinnen => 0.45,
        }
    }
}

// ---------------------------------------------------------------------------
// Tabel AA.2 (norm) / "Tijdstip maximale koellast" (xlsm)
// ---------------------------------------------------------------------------

/// Oriëntatie (γ) — 8 kompasrichtingen + horizontaal (dak).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Orientatie {
    /// Noord (γ = 360° / 0°).
    Noord,
    /// Noord-Oost (γ = 45°).
    NoordOost,
    /// Oost (γ = 90°).
    Oost,
    /// Zuid-Oost (γ = 135°).
    ZuidOost,
    /// Zuid (γ = 180°).
    Zuid,
    /// Zuid-West (γ = 225°).
    ZuidWest,
    /// West (γ = 270°).
    West,
    /// Noord-West (γ = 315°).
    NoordWest,
    /// Horizontaal (dak) — γ niet relevant.
    Horizontaal,
}

impl Orientatie {
    /// γ in graden (0..360). [`Orientatie::Horizontaal`] heeft geen γ,
    /// hier wordt 0° gerapporteerd (norm interpreteert β=0° als horizontaal
    /// waarbij γ irrelevant is — zie tabel AA.3 kolom β=0°).
    #[must_use]
    pub const fn gamma_deg(self) -> f64 {
        match self {
            Self::Noord => 0.0,
            Self::NoordOost => 45.0,
            Self::Oost => 90.0,
            Self::ZuidOost => 135.0,
            Self::Zuid => 180.0,
            Self::ZuidWest => 225.0,
            Self::West => 270.0,
            Self::NoordWest => 315.0,
            Self::Horizontaal => 0.0,
        }
    }

    /// Alle 8 windrichtingen in tabel-volgorde (γ = 0/360, 45, 90, …, 315).
    #[must_use]
    pub const fn windrichtingen() -> [Self; 8] {
        [
            Self::Noord,
            Self::NoordOost,
            Self::Oost,
            Self::ZuidOost,
            Self::Zuid,
            Self::ZuidWest,
            Self::West,
            Self::NoordWest,
        ]
    }
}

// ---------------------------------------------------------------------------
// Tabel AA.3 — I_sol matrix (W/m²) per β × γ × tijdstip
// ---------------------------------------------------------------------------

/// β-waarden (hellingshoek) waarvoor tabel AA.3 expliciete kolommen geeft.
/// Tussenliggende β wordt lineair geïnterpoleerd; buiten 0..180° → clamping.
pub const TABEL_AA_3_BETA_DEG: [f64; 7] = [0.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0];

/// Tijdstippen (uren) waarop tabel AA.3 waarden geeft (9..18 h).
pub const TABEL_AA_3_TIJDSTIPPEN: [u8; 10] = [9, 10, 11, 12, 13, 14, 15, 16, 17, 18];

/// Tabel AA.3 — `I_sol` [W/m²] per (β, γ, tijdstip).
///
/// Outer dimensie: β-index (zie [`TABEL_AA_3_BETA_DEG`]).
/// Middle dimensie: γ-index (Z=0, ZW=1, W=2, NW=3, N=4, NO=5, O=6, ZO=7) —
/// **let op**: kolom-volgorde komt overeen met de PDF/xlsm waar γ start
/// op 180° (Z), niet op 0° (N).
/// Inner dimensie: tijdstip-index (9 h = 0, 10 h = 1, …, 18 h = 9).
///
/// Voor β = 0° (horizontaal/platdak) zijn alle γ-kolommen identiek aan
/// `I_sol;mi` (eerste kolom in xlsm "Tabel AA"). De waarden zijn 1:1
/// overgenomen uit de RVO-rekentool 2025.04 zonder afronding.
#[allow(clippy::unreadable_literal)]
pub const TABEL_AA_3_I_SOL: [[[f64; 10]; 8]; 7] = [
    // β = 0° (horizontaal/plat dak) — alle oriëntaties krijgen I_sol;mi
    // (de "platdak" / `I_sol;mi`-kolom uit de xlsm).
    [
        // Z (γ=180°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // ZW (γ=225°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // W (γ=270°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // NW (γ=315°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // N (γ=360°/0°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // NO (γ=45°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // O (γ=90°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
        // ZO (γ=135°)
        [
            670.670107, 832.043291, 877.414162, 914.118562, 901.036454, 823.474921, 819.789825,
            601.563605, 446.488181, 308.218897,
        ],
    ],
    // β = 30°
    [
        // Z (γ=180°)
        [
            729.144231, 947.715946, 1026.060785, 1078.073647, 1056.998691, 1003.658488, 904.649897,
            608.839404, 403.007161, 202.751458,
        ],
        // ZW (γ=225°)
        [
            481.124655, 717.011742, 862.301498, 992.861352, 1061.118146, 1096.539739, 1098.243771,
            828.853388, 643.547749, 473.868743,
        ],
        // W (γ=270°)
        [
            307.220432, 505.251705, 655.990331, 807.671830, 918.515240, 981.937981, 1081.323342,
            882.281464, 745.900477, 633.308861,
        ],
        // NW (γ=315°)
        [
            309.302299, 436.481992, 527.981569, 630.986592, 712.724820, 726.985371, 863.800367,
            737.826190, 650.108504, 587.673953,
        ],
        // N (γ=360°/0°)
        [
            486.150726, 550.986969, 553.261007, 566.305453, 564.296124, 481.029690, 573.096856,
            480.107506, 412.285469, 363.696330,
        ],
        // NO (γ=45°)
        [
            734.170302, 781.691173, 717.020295, 651.517748, 560.176669, 388.148439, 379.502983,
            260.093522, 171.744881, 92.579045,
        ],
        // O (γ=90°)
        [
            908.074524, 993.451210, 923.331462, 836.707270, 702.779575, 502.750197, 396.423412,
            206.665446, 80.405901, 60.724567,
        ],
        // ZO (γ=135°)
        [
            905.992658, 1062.220923, 1051.340224, 1013.392508, 908.569995, 757.702807, 613.946386,
            351.120721, 165.184126, 60.724567,
        ],
    ],
    // β = 45°
    [
        // Z (γ=180°)
        [
            694.123489, 921.000359, 1007.949147, 1062.829081, 1039.729336, 1004.761878, 866.641921,
            559.348397, 346.985362, 133.028536,
        ],
        // ZW (γ=225°)
        [
            343.370840, 594.735345, 776.358541, 942.320699, 1045.555125, 1136.115801, 1140.425003,
            870.495157, 687.161125, 516.446278,
        ],
        // W (γ=270°)
        [
            97.433130, 295.261428, 484.590491, 680.423165, 843.884161, 974.044442, 1116.495902,
            946.053866, 831.909741, 741.928656,
        ],
        // NW (γ=315°)
        [
            100.377335, 198.006368, 303.558763, 430.552505, 552.852558, 613.487003, 808.871962,
            741.763258, 696.439434, 677.391150,
        ],
        // N (γ=360°/0°)
        [
            350.478778, 359.940859, 339.309289, 339.079561, 342.942683, 265.653143, 397.755114,
            377.294001, 360.106873, 360.638957,
        ],
        // NO (γ=45°)
        [
            701.231427, 686.205874, 570.899894, 459.587944, 337.116894, 134.299219, 123.972033,
            90.673939, 86.355439, 65.163807,
        ],
        // O (γ=90°)
        [
            947.169136, 985.679791, 862.667944, 721.485477, 538.787858, 296.370578, 147.901133,
            90.673939, 86.355439, 65.163807,
        ],
        // ZO (γ=135°)
        [
            944.224932, 1082.934851, 1043.699672, 971.356138, 829.819461, 656.928017, 455.525074,
            194.879139, 86.355439, 65.163807,
        ],
    ],
    // β = 60°
    [
        // Z (γ=180°)
        [
            618.469098, 839.048210, 928.437095, 982.656454, 959.061944, 945.059857, 777.235773,
            478.137935, 272.698118, 69.010667,
        ],
        // ZW (γ=225°)
        [
            188.886591, 439.456806, 644.797689, 835.064430, 966.197050, 1105.934902, 1112.550198,
            859.213333, 689.326638, 527.677278,
        ],
        // W (γ=270°)
        [
            103.499094, 106.711068, 287.456266, 514.306769, 719.201570, 907.438835, 1083.243155,
            951.753475, 866.606763, 803.835664,
        ],
        // NW (γ=315°)
        [
            103.499094, 106.711068, 99.499444, 208.278959, 362.762108, 465.847961, 706.482312,
            701.549600, 700.690199, 724.793685,
        ],
        // N (γ=360°/0°)
        [
            197.592001, 151.893465, 109.523858, 101.064187, 105.676065, 110.876281, 202.969061,
            255.167746, 288.768620, 336.853061,
        ],
        // NO (γ=45°)
        [
            627.174508, 551.484868, 393.163265, 243.839964, 101.181914, 110.876281, 110.660332,
            99.615902, 91.801028, 69.010667,
        ],
        // O (γ=90°)
        [
            928.385457, 918.264011, 750.504688, 564.597625, 345.536438, 110.876281, 110.660332,
            99.615902, 91.801028, 69.010667,
        ],
        // ZO (γ=135°)
        [
            924.779558, 1037.376648, 972.222368, 870.625435, 701.975901, 519.052121, 273.722522,
            99.615902, 91.801028, 69.010667,
        ],
    ],
    // β = 90° (gevel)
    [
        // Z (γ=180°)
        [
            368.300021, 535.157514, 610.701281, 653.433942, 633.468285, 663.380484, 471.687963,
            244.480549, 99.755563, 73.936852,
        ],
        // ZW (γ=225°)
        [
            125.306515, 138.428537, 283.182706, 483.009353, 641.707196, 849.142985, 858.875711,
            684.508516, 571.558432, 455.226551,
        ],
        // W (γ=270°)
        [
            125.306515, 138.428537, 137.901503, 141.665748, 356.501383, 619.939470, 825.034852,
            791.364668, 776.263887, 774.106787,
        ],
        // NW (γ=315°)
        [
            125.306515, 138.428537, 137.901503, 141.665748, 140.765718, 140.751685, 389.988904,
            502.454119, 584.679942, 682.836972,
        ],
        // N (γ=360°/0°)
        [
            125.306515, 138.428537, 137.901503, 141.665748, 140.765718, 140.751685, 140.134923,
            115.748650, 109.033872, 234.881725,
        ],
        // NO (γ=45°)
        [
            378.352162, 203.107968, 137.901503, 141.665748, 140.765718, 140.751685, 140.134923,
            115.748650, 99.755563, 73.936852,
        ],
        // O (γ=90°)
        [
            726.160607, 626.628042, 405.242634, 170.701187, 140.765718, 140.751685, 140.134923,
            115.748650, 99.755563, 73.936852,
        ],
        // ZO (γ=135°)
        [
            721.996873, 764.167468, 661.260159, 524.071664, 336.610893, 171.469121, 140.134923,
            115.748650, 99.755563, 73.936852,
        ],
    ],
    // β = 135°
    [
        // Z (γ=180°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            128.025436, 100.974261, 72.478809,
        ],
        // ZW (γ=225°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            152.552133, 167.398590, 160.421401,
        ],
        // W (γ=270°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            228.110843, 312.147206, 385.903778,
        ],
        // NW (γ=315°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            128.025436, 176.676899, 321.366273,
        ],
        // N (γ=360°/0°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            128.025436, 100.974261, 72.478809,
        ],
        // NO (γ=45°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            128.025436, 100.974261, 72.478809,
        ],
        // O (γ=90°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            128.025436, 100.974261, 72.478809,
        ],
        // ZO (γ=135°)
        [
            142.912829, 169.795847, 177.288815, 183.850774, 181.632820, 169.379749, 168.455362,
            128.025436, 100.974261, 72.478809,
        ],
    ],
    // β = 180° (vloer boven buitenlucht) — γ irrelevant, alle gelijk
    [
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
        [
            134.134021, 166.408658, 175.482832, 182.823712, 180.207291, 164.694984, 163.957965,
            120.312721, 89.297636, 61.643779,
        ],
    ],
];

/// Kolom-index voor γ in [`TABEL_AA_3_I_SOL`].
fn gamma_kolom_index(orientatie: Orientatie) -> usize {
    // Volgorde in tabel: 180° (Z), 225° (ZW), 270° (W), 315° (NW),
    // 360° (N), 45° (NO), 90° (O), 135° (ZO).
    match orientatie {
        Orientatie::Zuid | Orientatie::Horizontaal => 0,
        Orientatie::ZuidWest => 1,
        Orientatie::West => 2,
        Orientatie::NoordWest => 3,
        Orientatie::Noord => 4,
        Orientatie::NoordOost => 5,
        Orientatie::Oost => 6,
        Orientatie::ZuidOost => 7,
    }
}

/// Tijdstip-index voor `uur` in [`TABEL_AA_3_I_SOL`]. Geeft `None` voor uren
/// buiten 9..=18.
#[must_use]
pub fn tijdstip_index(uur: u8) -> Option<usize> {
    TABEL_AA_3_TIJDSTIPPEN.iter().position(|&h| h == uur)
}

/// Zoek I_sol in tabel AA.3 op (β, γ, tijdstip) met lineaire interpolatie
/// over β. γ wordt mapped naar exact één van de 8 windrichtingen of horizontaal.
///
/// β buiten 0..180° wordt geclamped op de tabelranden.
///
/// # Errors
///
/// Retourneert [`CoolingError::Model`] als `uur` niet in 9..=18 valt.
pub fn i_sol(
    beta_deg: f64,
    orientatie: Orientatie,
    uur: u8,
) -> CoolingCalcResult<f64> {
    let t_idx = tijdstip_index(uur).ok_or_else(|| {
        CoolingError::Model(nta8800_model::ModelError::OutOfRange {
            field: "uur (tabel AA.3 tijdstip)".into(),
            range: "9..=18".into(),
            value: uur.to_string(),
        })
    })?;
    let g_idx = gamma_kolom_index(orientatie);

    // β clampen op tabelranden 0..180°.
    let beta_clamped = beta_deg.clamp(TABEL_AA_3_BETA_DEG[0], TABEL_AA_3_BETA_DEG[6]);

    // Vind de twee β-indices waartussen we interpoleren.
    let (lo_idx, hi_idx) = match TABEL_AA_3_BETA_DEG
        .iter()
        .position(|&b| b >= beta_clamped)
    {
        Some(0) => (0, 0), // β = 0° exact
        Some(i) => (i - 1, i),
        // Onbereikbaar door clamping, maar wees expliciet.
        None => (6, 6),
    };

    let lo_val = TABEL_AA_3_I_SOL[lo_idx][g_idx][t_idx];
    let hi_val = TABEL_AA_3_I_SOL[hi_idx][g_idx][t_idx];

    if lo_idx == hi_idx {
        return Ok(lo_val);
    }

    let lo_beta = TABEL_AA_3_BETA_DEG[lo_idx];
    let hi_beta = TABEL_AA_3_BETA_DEG[hi_idx];
    let frac = (beta_clamped - lo_beta) / (hi_beta - lo_beta);
    Ok(lo_val + frac * (hi_val - lo_val))
}

// ---------------------------------------------------------------------------
// Tabel "Tijdstip maximale koellast" (xlsm) — informatieve hulp
// ---------------------------------------------------------------------------

/// Specifieke werkzame massa-klasse uit tabellen 7.11/7.12 NTA 8800. In
/// bijlage AA niet meer normatief gebruikt (norm geeft expliciet aan dat
/// SWM in deze methode geen rekenparameter is — AA.2 stap 1 opmerking).
/// Behouden als hulpfunctie voor backwards-compatibele rapportage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwmKlasse {
    /// SWM ≤ 50 kJ/m²K — tijdstip-tabel kolom "SWM = 50".
    Tot50,
    /// SWM > 50 kJ/m²K — tijdstip-tabel kolom "SWM > 50".
    Boven50,
}

// ---------------------------------------------------------------------------
// Invoer-types (Raam, Gevel, Ruimte)
// ---------------------------------------------------------------------------

/// Eén raam(type) in een verblijfsruimte voor de Bijlage AA-berekening.
/// Conform formule AA.6b/AA.7 worden de waardes per raam aangeleverd.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RaamAa {
    /// Oppervlakte raam (glas + kozijn) A_wi, in m².
    pub oppervlakte_m2: f64,
    /// g-waarde van het glas (g_gl;wi;C;juli), 0..=1.
    pub g_waarde: f64,
    /// U-waarde van het raam (U_w;wi), in W/(m²·K). Indien zonwering die
    /// voldoet aan 7.6.6.1.4 wordt door de caller `U_w+shut` verstrekt.
    pub u_waarde_w_per_m2k: f64,
    /// F_sh — beschaduwingsreductiefactor (overstek + zijbelemmering),
    /// 0..=1. Default 1.0 (geen belemmering).
    pub f_sh: f64,
    /// Zonwering-type — bepaalt F_C via [`ZonweringType::f_c`].
    pub zonwering: ZonweringType,
    /// Hellingshoek β van het raam, in graden (0=horizontaal/plat dak,
    /// 90=verticale gevel, 180=vloer-boven-buitenlucht).
    pub helling_beta_deg: f64,
    /// Oriëntatie γ van het raam.
    pub orientatie: Orientatie,
}

/// Eén verblijfsruimte (kamer) in de rekenzone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RuimteAa {
    /// Naam/label van de ruimte (voor rapportage).
    pub naam: String,
    /// Type ruimte — woonkamer/keuken/eetkamer telt 2× mee in AA.2/AA.3a;
    /// overige verblijfsruimten 1× (AA.3b).
    pub is_woonvertrek: bool,
    /// Gebruiksoppervlakte verblijfsruimte A_g;vr;zi,j, in m².
    pub oppervlakte_m2: f64,
    /// Binnenwerkse oppervlakte ondoorzichtig deel buitenwand + dak van de
    /// ruimte (voor AA.5), in m².
    pub opaque_oppervlakte_m2: f64,
    /// Ramen in deze ruimte.
    pub ramen: Vec<RaamAa>,
}

/// Top-level invoer voor de volledige bijlage AA-berekening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BijlageAaInput {
    /// Aantal woonfuncties N_woon in de rekenzone (NTA §6.6.6).
    pub aantal_woonfuncties: u32,
    /// Gemiddeld aantal bewoners per woonfunctie P_p;woon (7.5.2.1).
    pub bewoners_per_woonfunctie: f64,
    /// Bouwjaar van de rekenzone (drijft f_iso uit tabel AA.2).
    pub bouwjaar: u32,
    /// Infiltratie-luchtvolumestroom q_v;C;eff;lea;in;juli, in m³/h, voor
    /// de hele rekenzone (§11.2.1.7 NTA 8800).
    pub infiltratie_m3_per_h: f64,
    /// Natuurlijke toevoer-luchtvolumestroom q_v;C;eff;vent;in;juli, m³/h.
    pub natuurlijke_ventilatie_m3_per_h: f64,
    /// Mechanische toevoer-luchtvolumestroom q_v;C;SUP;eff;juli, m³/h.
    pub mechanische_ventilatie_m3_per_h: f64,
    /// Verblijfsruimten in de rekenzone. Bijlage AA stelt geen harde
    /// limiet aan het aantal — de RVO-rekentool ondersteunt max 10
    /// (tool-beperking, niet norm-beperking).
    pub ruimten: Vec<RuimteAa>,
}

// ---------------------------------------------------------------------------
// Resultaat-types
// ---------------------------------------------------------------------------

/// Resultaat per verblijfsruimte volgens AA.9/AA.13.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RuimteResultaatAa {
    /// Naam van de ruimte.
    pub naam: String,
    /// P_int;calc;vr;zi,j — interne warmtelast in deze ruimte (AA.3a/b), W.
    pub p_int_w: f64,
    /// P_V;vr;zi,j — naar oppervlakte gewogen aandeel buitenlucht (AA.4), W.
    pub p_v_w: f64,
    /// P_tr;ntr;vr;zi,j — transmissie ondoorzichtige delen (AA.5), W.
    pub p_tr_ntr_w: f64,
    /// P_sol;vr;zi,j — max zoninstraling (AA.6b), W.
    pub p_sol_w: f64,
    /// P_gl;vr;zi,j — transmissie via transparante delen (AA.7), W.
    pub p_gl_w: f64,
    /// Tijdstip (uur) waarop de maximale zoninstraling in deze ruimte
    /// optreedt (uitkomst van max-loop in AA.6b).
    pub maatgevend_tijdstip_uur: u8,
    /// q_C;vr;zi,j — maatgevende koelbehoefte (AA.9), W/m².
    pub q_c_w_per_m2: f64,
    /// B_C;req;TO;vr;zi,j — benodigde koelcapaciteit afgifte (AA.13), kW (≥0).
    pub b_c_req_kw: f64,
}

/// Resultaat op rekenzone-niveau volgens AA.8/AA.11.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BijlageAaResult {
    /// Per-ruimte resultaten (zelfde volgorde als input).
    pub ruimten: Vec<RuimteResultaatAa>,
    /// N_int;zi — basiswaarde interne warmtelast (AA.1), W.
    pub n_int_w: f64,
    /// q_int;calc;zi — rekenwaarde interne warmtelast (AA.2), W/m².
    pub q_int_calc_w_per_m2: f64,
    /// P_int;calc;zi — sommatie interne warmtelast over zone, W.
    pub p_int_zone_w: f64,
    /// P_V;zi — buitenluchttoetreding rekenzone (AA.4), W.
    pub p_v_zone_w: f64,
    /// P_tr;ntr;zi — transmissie ondoorzichtig rekenzone (AA.5), W.
    pub p_tr_ntr_zone_w: f64,
    /// P_sol;zi — sommatie zoninstraling alle verblijfsruimten (AA.6a), W.
    pub p_sol_zone_w: f64,
    /// P_gl;zi — transmissie via glas op maatgevend zone-tijdstip (AA.7), W.
    /// In V1 berekend op het tijdstip met de grootste sommatie van P_sol
    /// over alle ruimten — zie [`BijlageAaResult::maatgevend_tijdstip_uur`].
    pub p_gl_zone_w: f64,
    /// Maatgevend tijdstip voor de rekenzone — het uur waarop de som van
    /// P_sol over alle verblijfsruimten maximaal is.
    pub maatgevend_tijdstip_uur: u8,
    /// q_C;zi — maatgevende koelbehoefte rekenzone (AA.8), W/m².
    pub q_c_zone_w_per_m2: f64,
    /// B_C;req;TO;zi — benodigde koelcapaciteit opwekker rekenzone (AA.11), kW.
    pub b_c_req_zone_kw: f64,
    /// Totale verblijfsruimte-oppervlakte A_g;vr;zi in de rekenzone, m².
    pub totaal_oppervlakte_m2: f64,
}

// ---------------------------------------------------------------------------
// Formule-implementaties
// ---------------------------------------------------------------------------

/// Formule (AA.1) — basiswaarde interne warmtelast `N_int;zi`, W.
///
/// `N_int;zi = 180 · N_woon · P_p;woon`
///
/// # Errors
/// [`CoolingError::InvalidPersonCount`] als `bewoners_per_woonfunctie ≤ 0`.
pub fn formule_aa1_n_int(
    aantal_woonfuncties: u32,
    bewoners_per_woonfunctie: f64,
) -> CoolingCalcResult<f64> {
    if !bewoners_per_woonfunctie.is_finite() || bewoners_per_woonfunctie <= 0.0 {
        return Err(CoolingError::InvalidPersonCount {
            persons: bewoners_per_woonfunctie,
        });
    }
    Ok(180.0 * f64::from(aantal_woonfuncties) * bewoners_per_woonfunctie)
}

/// Formule (AA.2) — rekenwaarde interne warmtelast `q_int;calc;zi`, W/m².
///
/// `q_int;calc;zi = N_int / (2 · A_vr;woon + A_vr;overig)`
///
/// # Errors
/// [`CoolingError::InvalidFloorArea`] als `2·A_woon + A_overig ≤ 0`.
pub fn formule_aa2_q_int_calc(
    n_int_w: f64,
    a_woon_m2: f64,
    a_overig_m2: f64,
) -> CoolingCalcResult<f64> {
    let noemer = 2.0 * a_woon_m2 + a_overig_m2;
    if !noemer.is_finite() || noemer <= 0.0 {
        return Err(CoolingError::InvalidFloorArea { area_m2: noemer });
    }
    Ok(n_int_w / noemer)
}

/// Formule (AA.3a) — interne warmtelast woonkamer/keuken/eetkamer, W.
/// `P_int;calc;woon = 2 · q_int;calc · A_vg;woon`.
#[must_use]
pub fn formule_aa3a_p_int_woon(q_int_calc: f64, a_woon_m2: f64) -> f64 {
    2.0 * q_int_calc * a_woon_m2
}

/// Formule (AA.3b) — interne warmtelast overige verblijfsruimte, W.
/// `P_int;calc;overig = q_int;calc · A_vg;overig`.
#[must_use]
pub fn formule_aa3b_p_int_overig(q_int_calc: f64, a_overig_m2: f64) -> f64 {
    q_int_calc * a_overig_m2
}

/// Formule (AA.4) — koellast door buitenluchttoetreding, W.
///
/// `P_V = ((q_v;lea + q_v;vent + q_v;mech) / 3600) · ρ_a · c_a · (θ_e − 24)`
///
/// Clamped op 0 als θ_e ≤ 24 °C (norm: P_V kan niet negatief zijn).
#[must_use]
pub fn formule_aa4_p_v(
    q_v_lea_m3_per_h: f64,
    q_v_vent_m3_per_h: f64,
    q_v_mech_m3_per_h: f64,
    theta_e_c: f64,
) -> f64 {
    let totaal_m3_per_s = (q_v_lea_m3_per_h + q_v_vent_m3_per_h + q_v_mech_m3_per_h) / 3600.0;
    let delta_t = theta_e_c - INDOOR_TEMPERATURE_C;
    if delta_t <= 0.0 {
        return 0.0;
    }
    totaal_m3_per_s * RHO_AIR_KG_PER_M3 * C_AIR_J_PER_KGK * delta_t
}

/// Formule (AA.5) — koellast transmissie ondoorzichtige delen, W.
/// `P_tr;ntr = f_iso · A_in`.
#[must_use]
pub fn formule_aa5_p_tr_ntr(klasse: BouwjaarKlasseAa, a_in_m2: f64) -> f64 {
    klasse.f_iso() * a_in_m2
}

/// Formule (AA.6b) — koellast door zoninstraling per verblijfsruimte, W.
///
/// `P_sol;vr;j = max over t=9..18h van Σ (0,75 · A_wi · g_gl · F_sh · F_C · I_sol;t)`
///
/// Retourneert (P_sol max, maatgevend tijdstip).
///
/// # Errors
/// [`CoolingError::Model`] als interne tabel-lookup faalt (zou niet moeten
/// gebeuren omdat we de tijdstippen zelf controleren).
pub fn formule_aa6b_p_sol_ruimte(ramen: &[RaamAa]) -> CoolingCalcResult<(f64, u8)> {
    let mut maxima: Option<(f64, u8)> = None;
    for &uur in &TABEL_AA_3_TIJDSTIPPEN {
        let mut som = 0.0_f64;
        for raam in ramen {
            let i_sol_val = i_sol(raam.helling_beta_deg, raam.orientatie, uur)?;
            let f_c = raam.zonwering.f_c();
            som += RAAM_GLASVLAK_RATIO
                * raam.oppervlakte_m2
                * raam.g_waarde
                * raam.f_sh
                * f_c
                * i_sol_val;
        }
        match maxima {
            None => maxima = Some((som, uur)),
            Some((huidig, _)) if som > huidig => maxima = Some((som, uur)),
            _ => {}
        }
    }
    // Bij lege ramen-lijst: 0 W om 9 h (eerste tijdstip).
    Ok(maxima.unwrap_or((0.0, 9)))
}

/// Formule (AA.7) — koellast transmissie via transparante delen, W.
/// `P_gl;zi = Σ (A_wi · U_w;wi) · (θ_e − 24)`.
/// θ_e wordt afgeleid uit tabel AA.1 op het opgegeven tijdstip.
///
/// # Errors
/// [`CoolingError::Model`] als `uur` niet in 9..=21 valt (tabel AA.1 bereik).
pub fn formule_aa7_p_gl(ramen: &[RaamAa], uur: u8) -> CoolingCalcResult<f64> {
    let theta = theta_e(uur).ok_or_else(|| {
        CoolingError::Model(nta8800_model::ModelError::OutOfRange {
            field: "uur (tabel AA.1)".into(),
            range: "9..=21".into(),
            value: uur.to_string(),
        })
    })?;
    let delta = theta - INDOOR_TEMPERATURE_C;
    let som_au: f64 = ramen
        .iter()
        .map(|r| r.oppervlakte_m2 * r.u_waarde_w_per_m2k)
        .sum();
    Ok(som_au * delta)
}

/// Formule (AA.8) — maatgevende koelbehoefte rekenzone q_C;zi, W/m².
///
/// `q_C;zi = (P_int + P_V + P_tr;ntr + P_sol + P_gl) / A_g;vr;zi`
///
/// # Errors
/// [`CoolingError::InvalidFloorArea`] als oppervlakte ≤ 0.
pub fn formule_aa8_q_c_zone(
    p_int_w: f64,
    p_v_w: f64,
    p_tr_ntr_w: f64,
    p_sol_w: f64,
    p_gl_w: f64,
    a_g_vr_m2: f64,
) -> CoolingCalcResult<f64> {
    if !a_g_vr_m2.is_finite() || a_g_vr_m2 <= 0.0 {
        return Err(CoolingError::InvalidFloorArea {
            area_m2: a_g_vr_m2,
        });
    }
    Ok((p_int_w + p_v_w + p_tr_ntr_w + p_sol_w + p_gl_w) / a_g_vr_m2)
}

/// Formules (AA.11)/(AA.13) — benodigde koelcapaciteit, kW (≥0).
///
/// `B_C;req;TO = max(0, (q_C − 35) / 1000 · A)`
#[must_use]
pub fn formule_aa11_b_c_req(q_c_w_per_m2: f64, a_m2: f64) -> f64 {
    let raw = (q_c_w_per_m2 - FIXED_DEDUCTION_W_PER_M2) / 1000.0 * a_m2;
    if raw < 0.0 {
        0.0
    } else {
        raw
    }
}

// ---------------------------------------------------------------------------
// Top-level orchestratie
// ---------------------------------------------------------------------------

/// Tijdelijke intermediair per ruimte tijdens orchestratie.
struct RuimteIntermediair {
    idx: usize,
    p_int: f64,
    p_sol: f64,
    tijdstip_sol: u8,
    p_tr_ntr: f64,
    p_sol_per_uur: [f64; 10],
}

/// Voer een complete bijlage AA-berekening uit voor één rekenzone.
///
/// Implementeert formules AA.1 t/m AA.13 in onderstaande volgorde:
/// 1. AA.1: `N_int;zi`
/// 2. AA.2: `q_int;calc;zi` (verdeling woon vs overig)
/// 3. AA.3a/b: `P_int;calc;woon` en `P_int;calc;overig` per ruimte
/// 4. AA.6b: `P_sol;vr;j` per ruimte (max over 9..18 h via tabel AA.3)
/// 5. AA.6a: `P_sol;zi = Σ P_sol;vr` over rekenzone
/// 6. Bepaal **maatgevend tijdstip rekenzone** = uur waarop Σ P_sol over
///    alle ruimten maximaal is (conservatieve aanname conform AA.8 opm. 2).
/// 7. AA.1 θ_e lookup → AA.4: `P_V;zi` voor de rekenzone, daarna naar rato
///    van oppervlakte verdeeld over ruimten.
/// 8. AA.5: `P_tr;ntr;zi` voor zone, en per ruimte op basis van eigen
///    `opaque_oppervlakte_m2`.
/// 9. AA.7: `P_gl;zi` voor de rekenzone op het maatgevend tijdstip,
///    en `P_gl;vr;j` per ruimte op haar eigen maatgevend tijdstip.
/// 10. AA.8: `q_C;zi`
/// 11. AA.9: `q_C;vr;zi,j` per ruimte
/// 12. AA.11: `B_C;req;TO;zi`
/// 13. AA.13: `B_C;req;TO;vr;zi,j` per ruimte
///
/// # Errors
/// Diverse [`CoolingError`] varianten — zie individuele formules.
pub fn calculate_bijlage_aa(input: &BijlageAaInput) -> CoolingCalcResult<BijlageAaResult> {
    if input.ruimten.is_empty() {
        return Err(CoolingError::InvalidFloorArea { area_m2: 0.0 });
    }

    // Splitsing woon vs overig (AA.2-noemer)
    let a_woon: f64 = input
        .ruimten
        .iter()
        .filter(|r| r.is_woonvertrek)
        .map(|r| r.oppervlakte_m2)
        .sum();
    let a_overig: f64 = input
        .ruimten
        .iter()
        .filter(|r| !r.is_woonvertrek)
        .map(|r| r.oppervlakte_m2)
        .sum();
    let a_totaal = a_woon + a_overig;
    if !a_totaal.is_finite() || a_totaal <= 0.0 {
        return Err(CoolingError::InvalidFloorArea { area_m2: a_totaal });
    }

    // AA.1 + AA.2
    let n_int = formule_aa1_n_int(
        input.aantal_woonfuncties,
        input.bewoners_per_woonfunctie,
    )?;
    let q_int_calc = formule_aa2_q_int_calc(n_int, a_woon, a_overig)?;

    // AA.3a/b + AA.6b per ruimte
    let klasse = BouwjaarKlasseAa::from_year(input.bouwjaar);
    let mut intermediairs: Vec<RuimteIntermediair> = Vec::with_capacity(input.ruimten.len());

    for (idx, ruimte) in input.ruimten.iter().enumerate() {
        // AA.3a/b
        let p_int = if ruimte.is_woonvertrek {
            formule_aa3a_p_int_woon(q_int_calc, ruimte.oppervlakte_m2)
        } else {
            formule_aa3b_p_int_overig(q_int_calc, ruimte.oppervlakte_m2)
        };

        // AA.6b: max over 9..18 h
        let (p_sol_max, tijdstip_max) = formule_aa6b_p_sol_ruimte(&ruimte.ramen)?;

        // Bewaar volledige tijdsreeks voor zone-maatgevend tijdstip
        let mut p_sol_per_uur = [0.0_f64; 10];
        for (t_idx, &uur) in TABEL_AA_3_TIJDSTIPPEN.iter().enumerate() {
            let mut som = 0.0_f64;
            for raam in &ruimte.ramen {
                let i_sol_val = i_sol(raam.helling_beta_deg, raam.orientatie, uur)?;
                som += RAAM_GLASVLAK_RATIO
                    * raam.oppervlakte_m2
                    * raam.g_waarde
                    * raam.f_sh
                    * raam.zonwering.f_c()
                    * i_sol_val;
            }
            p_sol_per_uur[t_idx] = som;
        }

        // AA.5 per ruimte
        let p_tr_ntr = formule_aa5_p_tr_ntr(klasse, ruimte.opaque_oppervlakte_m2);

        intermediairs.push(RuimteIntermediair {
            idx,
            p_int,
            p_sol: p_sol_max,
            tijdstip_sol: tijdstip_max,
            p_tr_ntr,
            p_sol_per_uur,
        });
    }

    // AA.6a: som P_sol over zone (conservatief, ruimten kunnen verschillende
    // maatgevende tijdstippen hebben — AA.8 opmerking 2)
    let p_sol_zone: f64 = intermediairs.iter().map(|x| x.p_sol).sum();

    // Bepaal maatgevend tijdstip rekenzone = uur waarop Σ P_sol(uur) over alle
    // ruimten maximaal is (basis voor AA.7 θ_e lookup op zone-niveau).
    let mut beste_uur_idx = 0_usize;
    let mut beste_som = f64::NEG_INFINITY;
    for t_idx in 0..TABEL_AA_3_TIJDSTIPPEN.len() {
        let som: f64 = intermediairs.iter().map(|x| x.p_sol_per_uur[t_idx]).sum();
        if som > beste_som {
            beste_som = som;
            beste_uur_idx = t_idx;
        }
    }
    let maatgevend_uur_zone = TABEL_AA_3_TIJDSTIPPEN[beste_uur_idx];
    let theta_e_zone = theta_e(maatgevend_uur_zone).ok_or_else(|| {
        CoolingError::Model(nta8800_model::ModelError::OutOfRange {
            field: "maatgevend_uur_zone".into(),
            range: "9..=21".into(),
            value: maatgevend_uur_zone.to_string(),
        })
    })?;

    // AA.4: zone-totaal en verdeling naar ruimten naar rato van oppervlak
    let p_v_zone = formule_aa4_p_v(
        input.infiltratie_m3_per_h,
        input.natuurlijke_ventilatie_m3_per_h,
        input.mechanische_ventilatie_m3_per_h,
        theta_e_zone,
    );

    // AA.5 zone-totaal
    let p_tr_ntr_zone: f64 = intermediairs.iter().map(|x| x.p_tr_ntr).sum();

    // AA.7 voor zone op maatgevend uur
    let alle_ramen: Vec<RaamAa> = input
        .ruimten
        .iter()
        .flat_map(|r| r.ramen.iter().copied())
        .collect();
    let p_gl_zone = formule_aa7_p_gl(&alle_ramen, maatgevend_uur_zone)?;

    // P_int op zone-niveau (AA.3a + AA.3b sommatie)
    let p_int_zone: f64 = intermediairs.iter().map(|x| x.p_int).sum();

    // AA.8
    let q_c_zone = formule_aa8_q_c_zone(
        p_int_zone,
        p_v_zone,
        p_tr_ntr_zone,
        p_sol_zone,
        p_gl_zone,
        a_totaal,
    )?;

    // AA.11
    let b_c_req_zone = formule_aa11_b_c_req(q_c_zone, a_totaal);

    // Per-ruimte resultaten (AA.9 + AA.13)
    let mut ruimte_resultaten: Vec<RuimteResultaatAa> = Vec::with_capacity(input.ruimten.len());
    for inter in &intermediairs {
        let ruimte = &input.ruimten[inter.idx];

        // AA.4 naar rato: P_V;vr;j = P_V;zi · (A_vr;j / A_g;vr;zi)
        let p_v_ruimte = p_v_zone * (ruimte.oppervlakte_m2 / a_totaal);

        // AA.7 per ruimte op haar eigen maatgevend tijdstip
        let p_gl_ruimte = formule_aa7_p_gl(&ruimte.ramen, inter.tijdstip_sol)?;

        // AA.9
        let q_c_ruimte = formule_aa8_q_c_zone(
            inter.p_int,
            p_v_ruimte,
            inter.p_tr_ntr,
            inter.p_sol,
            p_gl_ruimte,
            ruimte.oppervlakte_m2,
        )?;

        // AA.13
        let b_c_req_ruimte = formule_aa11_b_c_req(q_c_ruimte, ruimte.oppervlakte_m2);

        ruimte_resultaten.push(RuimteResultaatAa {
            naam: ruimte.naam.clone(),
            p_int_w: inter.p_int,
            p_v_w: p_v_ruimte,
            p_tr_ntr_w: inter.p_tr_ntr,
            p_sol_w: inter.p_sol,
            p_gl_w: p_gl_ruimte,
            maatgevend_tijdstip_uur: inter.tijdstip_sol,
            q_c_w_per_m2: q_c_ruimte,
            b_c_req_kw: b_c_req_ruimte,
        });
    }

    Ok(BijlageAaResult {
        ruimten: ruimte_resultaten,
        n_int_w: n_int,
        q_int_calc_w_per_m2: q_int_calc,
        p_int_zone_w: p_int_zone,
        p_v_zone_w: p_v_zone,
        p_tr_ntr_zone_w: p_tr_ntr_zone,
        p_sol_zone_w: p_sol_zone,
        p_gl_zone_w: p_gl_zone,
        maatgevend_tijdstip_uur: maatgevend_uur_zone,
        q_c_zone_w_per_m2: q_c_zone,
        b_c_req_zone_kw: b_c_req_zone,
        totaal_oppervlakte_m2: a_totaal,
    })
}

// ---------------------------------------------------------------------------
// Unit tests (intern — uitgebreidere tests in tests/bijlage_aa_test.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn tabel_aa1_volledig() {
        assert_eq!(theta_e(9), Some(24.7));
        assert_eq!(theta_e(17), Some(30.6));
        assert_eq!(theta_e(21), Some(23.4));
        assert_eq!(theta_e(8), None);
        assert_eq!(theta_e(22), None);
    }

    #[test]
    fn tabel_aa3_beta0_uniform_over_gamma() {
        // β = 0° (horizontaal): alle oriëntaties moeten dezelfde I_sol;mi geven
        let z = i_sol(0.0, Orientatie::Zuid, 12).unwrap();
        let n = i_sol(0.0, Orientatie::Noord, 12).unwrap();
        let o = i_sol(0.0, Orientatie::Oost, 12).unwrap();
        assert_abs_diff_eq!(z, n, epsilon = 1e-9);
        assert_abs_diff_eq!(z, o, epsilon = 1e-9);
    }

    #[test]
    fn tabel_aa3_beta30_zuid_12h_komt_overeen_met_xlsm() {
        // Volgens "Tabel AA" rij β=30°, kolom Z (180°), tijdstip 12 h:
        // 1078.073647 W/m²
        let v = i_sol(30.0, Orientatie::Zuid, 12).unwrap();
        assert_abs_diff_eq!(v, 1078.073647, epsilon = 1e-3);
    }

    #[test]
    fn tabel_aa3_beta45_oost_10h_komt_overeen_met_xlsm() {
        // Volgens xlsm: β=45°, γ=90° (O), 10 h → 985.679791
        let v = i_sol(45.0, Orientatie::Oost, 10).unwrap();
        assert_abs_diff_eq!(v, 985.679791, epsilon = 1e-3);
    }

    #[test]
    fn interpolatie_beta375_ligt_lineair_tussen_30_en_45() {
        // β = 37.5° ligt halverwege tussen 30° en 45°
        let lo = i_sol(30.0, Orientatie::Zuid, 13).unwrap();
        let hi = i_sol(45.0, Orientatie::Zuid, 13).unwrap();
        let mid = i_sol(37.5, Orientatie::Zuid, 13).unwrap();
        assert_abs_diff_eq!(mid, (lo + hi) / 2.0, epsilon = 1e-9);
    }

    #[test]
    fn interpolatie_clamp_buiten_bereik() {
        // beta = -10 en beta = 200 clampen op respectievelijk 0 en 180
        let onder = i_sol(-10.0, Orientatie::Zuid, 12).unwrap();
        let ref0 = i_sol(0.0, Orientatie::Zuid, 12).unwrap();
        assert_abs_diff_eq!(onder, ref0, epsilon = 1e-9);

        let boven = i_sol(200.0, Orientatie::Zuid, 12).unwrap();
        let ref180 = i_sol(180.0, Orientatie::Zuid, 12).unwrap();
        assert_abs_diff_eq!(boven, ref180, epsilon = 1e-9);
    }

    #[test]
    fn i_sol_uur_buiten_bereik_geeft_error() {
        let err = i_sol(30.0, Orientatie::Zuid, 8).unwrap_err();
        assert!(matches!(err, CoolingError::Model(_)));
        let err = i_sol(30.0, Orientatie::Zuid, 19).unwrap_err();
        assert!(matches!(err, CoolingError::Model(_)));
    }

    #[test]
    fn bouwjaarklasse_grenzen() {
        assert_eq!(BouwjaarKlasseAa::from_year(1974), BouwjaarKlasseAa::Tot1975);
        assert_eq!(
            BouwjaarKlasseAa::from_year(1975),
            BouwjaarKlasseAa::Van1975Tot1992
        );
        assert_eq!(
            BouwjaarKlasseAa::from_year(2014),
            BouwjaarKlasseAa::Van1992Tot2015
        );
        assert_eq!(BouwjaarKlasseAa::from_year(2015), BouwjaarKlasseAa::Van2015);
    }

    #[test]
    fn aa1_basis() {
        // 1 woning × 3 bewoners × 180 = 540 W
        let n = formule_aa1_n_int(1, 3.0).unwrap();
        assert_abs_diff_eq!(n, 540.0, epsilon = 1e-9);
    }

    #[test]
    fn aa4_voorbeeld_juli_17h() {
        // 250 m³/h totaal, θ_e=30.6°C → (250/3600)·1.205·1005·6.6 ≈ 554.4 W
        let p = formule_aa4_p_v(100.0, 0.0, 150.0, 30.6);
        assert!(p > 500.0 && p < 600.0);
    }

    #[test]
    fn aa11_clamping_op_nul() {
        // q_C = 20 W/m² < 35 → 0 kW
        let b = formule_aa11_b_c_req(20.0, 120.0);
        assert_abs_diff_eq!(b, 0.0, epsilon = 1e-9);
    }
}
