//! NTA 8800 bijlage X — significante cijfers voor klassenindeling.
//!
//! Implementeert de afrondingsregels uit bijlage X:
//!
//! - Tabel X.1 geeft 33 toegestane basiswaarden (10, 11, ..., 20, 22, 24, ...,
//!   40, 44, 48, 52, 56, 60, 65, 70, 75, 80, 85, 90, 95).
//! - §X.2 beschrijft "naar boven" en "naar beneden" afronden op 2 significante
//!   cijfers waarbij eerst gezocht wordt naar het passende twee-cijferige
//!   patroon in tabel X.1, en vervolgens alle daaropvolgende cijfers op nul
//!   worden gezet.
//! - Het getal 0 wordt **nooit** afgerond.
//!
//! Bron: PDF p. 1129-1130 van NTA 8800:2025+C1:2026.
//!
//! # Gebruik
//!
//! Deze module levert een minimale API: [`round_to_significant_figures`]. Voor
//! specifieke grootheden zoals `H_sto;ls`, `P_sol;pmp`, `Q_H;ren` geeft de norm
//! per paragraaf expliciet aan of naar boven of naar beneden moet worden
//! afgerond — zie de referenties in [`crate::references`] en de thema-crates.

/// Toegestane basiswaarden uit tabel X.1 — strict stijgend gesorteerd.
///
/// Elke waarde is een twee-cijferig patroon (zonder voorloopnullen) dat bij
/// afronden op twee significante cijfers mag worden gebruikt. Bijvoorbeeld:
/// 0,33 rondt naar boven af op 0,34 (basispatroon `34`), en 3 327 rondt naar
/// boven af op 3 400 (basispatroon `34`).
///
/// Bron: [`NTA_8800_2025_BIJLAGE_X_TABEL1`](crate::references::NTA_8800_2025_BIJLAGE_X_TABEL1)
/// (PDF p. 1129).
pub const TABEL_X1_BASIS: [u32; 33] = [
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 44, 48, 52,
    56, 60, 65, 70, 75, 80, 85, 90, 95,
];

/// Richting waarin naar twee significante cijfers moet worden afgerond.
///
/// Bijlage X schrijft per paragraaf voor of een grootheid naar boven of naar
/// beneden wordt afgerond (bv. vermogens naar boven, hernieuwbare energie
/// `Q_W;ren` naar beneden). Deze enum bewaart die keuze type-veilig.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingDirection {
    /// Naar boven afronden — eerstvolgende **hogere** waarde uit tabel X.1.
    ///
    /// Gebruikt voor o.a. vermogens van installatiecomponenten `H_sto;ls`,
    /// pompvermogens `P_sol;pmp` en parasitaire verlichtingsvermogens.
    Up,
    /// Naar beneden afronden — eerstvolgende **lagere** waarde uit tabel X.1
    /// (of de waarde zelf als die al in de tabel staat).
    ///
    /// Gebruikt voor o.a. hernieuwbare warmtebijdragen `Q_W;ren;si,mi` en
    /// `Q_H;ren;si,mi`.
    Down,
}

/// Afrondingsregel per grootheid volgens NTA 8800 bijlage X.
///
/// Elke variant combineert een grootheid met de norm-voorgeschreven afrondings-
/// richting en minimale kenmerken voor traceability. De onderliggende
/// rekenregel is altijd: pas tabel X.1 toe op de eerste twee significante
/// cijfers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingRule {
    /// Vermogen van een installatiecomponent in W of kW — afronden **naar boven**
    /// op 2 significante cijfers (bv. `H_sto;ls`, `P_sol;pmp`, `P_n;spec`).
    ///
    /// Referentie: NTA 8800 §13.6 / §13.7 / §14.2.
    VermogenInstallatiecomponent,
    /// Opwekkingsrendement van een voorraadvat of vergelijkbaar stand-byverlies
    /// — afronden **naar boven** op 2 significante cijfers.
    ///
    /// Referentie: NTA 8800 §13.6.
    StandByVerlies,
    /// Energiebijdrage van hernieuwbare bronnen aan tapwater of ruimteverwarming
    /// `Q_W;ren;si,mi`, `Q_H;ren;si,mi` — afronden **naar beneden** op 2
    /// significante cijfers.
    ///
    /// Referentie: NTA 8800 §13.7 / §13.8.
    HernieuwbareEnergieBijdrage,
    /// Energie geleverd door BWP ter compensatie van warmteverlies
    /// `Q_W;BWP;ls;tot;mi` — afronden **naar beneden** op 2 significante cijfers.
    ///
    /// Referentie: NTA 8800 §13.8.4.4.
    BwpWarmteverliesCompensatie,
    /// Collector-oppervlakte-afhankelijke klassegrens (zonneboiler) — afronden
    /// **naar beneden** op 2 significante cijfers.
    ///
    /// Referentie: NTA 8800 §13.7.2.2.
    ZonneboilerKlasse,
    /// Parasitair vermogen voor verlichting `W_p` — afronden **naar boven** op
    /// 2 significante cijfers.
    ///
    /// Referentie: NTA 8800 §14.2 (formule 14.10).
    ParasitairVermogenVerlichting,
}

impl RoundingRule {
    /// Geef de afrondingsrichting voor deze regel.
    #[must_use]
    pub const fn direction(self) -> RoundingDirection {
        match self {
            RoundingRule::VermogenInstallatiecomponent
            | RoundingRule::StandByVerlies
            | RoundingRule::ParasitairVermogenVerlichting => RoundingDirection::Up,
            RoundingRule::HernieuwbareEnergieBijdrage
            | RoundingRule::BwpWarmteverliesCompensatie
            | RoundingRule::ZonneboilerKlasse => RoundingDirection::Down,
        }
    }
}

/// Rond een waarde af op 2 significante cijfers volgens NTA 8800 bijlage X.
///
/// - Retourneert `0.0` zodra de invoer `0.0` is — het getal 0 wordt nooit
///   afgerond (§X.2).
/// - Retourneert `value` onveranderd bij niet-eindige invoer (`NaN`, `±∞`).
/// - Behoudt het teken: negatieve invoer wordt in absolute waarde afgerond
///   en daarna terug voorzien van het originele teken.
///
/// # Algoritme (§X.2)
///
/// 1. Neem de absolute waarde `|v|`.
/// 2. Bepaal de exponent `e = ⌊log₁₀(|v|)⌋ - 1` zodat `|v| / 10^e` tussen 10
///    en 100 ligt (de "eerste twee significante cijfers" als integer).
/// 3. `basis_f = |v| / 10^e` (float).
/// 4. Zoek in [`TABEL_X1_BASIS`] de juiste waarde:
///    - Bij [`RoundingDirection::Up`]: neem de eerstvolgende waarde `>= ⌈basis_f⌉`.
///      Als de float al exact gelijk is aan een tabelwaarde én het restgedeelte
///      is 0, dan blijft die waarde staan.
///    - Bij [`RoundingDirection::Down`]: neem de grootste tabelwaarde
///      `<= ⌊basis_f⌋`.
/// 5. Vermenigvuldig terug met `10^e` en herstel het teken.
///
/// # Voorbeelden (uit de OPMERKINGEN bij tabel X.1)
///
/// ```
/// use nta8800_tables::rounding::{round_to_significant_figures, RoundingRule};
///
/// // Naar boven: vermogens
/// let r = RoundingRule::VermogenInstallatiecomponent;
/// assert!((round_to_significant_figures(0.33, r) - 0.34).abs() < 1e-12);
/// assert!((round_to_significant_figures(3.3, r) - 3.4).abs() < 1e-12);
/// assert!((round_to_significant_figures(33.0, r) - 34.0).abs() < 1e-12);
/// assert!((round_to_significant_figures(3327.0, r) - 3400.0).abs() < 1e-9);
/// assert!((round_to_significant_figures(3450.0, r) - 3600.0).abs() < 1e-9);
///
/// // Naar beneden: hernieuwbare energie-bijdragen
/// let d = RoundingRule::HernieuwbareEnergieBijdrage;
/// assert!((round_to_significant_figures(0.33, d) - 0.32).abs() < 1e-12);
/// assert!((round_to_significant_figures(3450.0, d) - 3400.0).abs() < 1e-9);
/// ```
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]
pub fn round_to_significant_figures(value: f64, rule: RoundingRule) -> f64 {
    if !value.is_finite() {
        return value;
    }
    // Het getal 0 wordt nooit afgerond (§X.2). Exacte float-vergelijking is
    // hier correct: invoer 0.0 moet exact 0.0 blijven.
    if value == 0.0 {
        return 0.0;
    }

    let sign = value.signum();
    let abs = value.abs();

    // Bepaal exponent zodat basis_f = abs / 10^exp ∈ [10, 100).
    //
    // Voorbeeld: abs = 3327  ⇒ log10 = 3.522, floor=3, exp=2, basis_f=33.27.
    // Voorbeeld: abs = 0.33  ⇒ log10 = -0.481, floor=-1, exp=-2, basis_f=33.0.
    //
    // `as i32` is veilig: voor elk eindig, niet-nul, niet-subnormaal f64 ligt
    // log10 in [-308, 308] — ruim binnen i32-bereik.
    let log10 = abs.log10();
    let exp = log10.floor() as i32 - 1;
    let scale = 10f64.powi(exp);
    let basis_f = abs / scale;

    let basis = match rule.direction() {
        RoundingDirection::Up => round_up_tabel_x1(basis_f),
        RoundingDirection::Down => round_down_tabel_x1(basis_f),
    };

    sign * f64::from(basis) * scale
}

/// Zoek het eerstvolgende strict-hogere patroon in [`TABEL_X1_BASIS`].
///
/// §X.2: "neem het **eerstvolgende hogere** getal uit tabel X.1". Dit is
/// strict greater-than — ook bij exacte match gaat de waarde een stap omhoog
/// (bv. `3 400 → 3 600`, `33 → 34`).
///
/// Als `basis_f >= 95.0` (wat niet kan bij correcte exponent-bepaling, maar
/// defensief toch afgevangen): retourneer 95 als fallback-bovengrens.
fn round_up_tabel_x1(basis_f: f64) -> u32 {
    for &v in &TABEL_X1_BASIS {
        if f64::from(v) > basis_f {
            return v;
        }
    }
    *TABEL_X1_BASIS.last().unwrap_or(&95)
}

/// Zoek het eerstvolgende lagere (of gelijke, bij exacte match) patroon in
/// [`TABEL_X1_BASIS`].
///
/// Als `basis_f < 10.0` (niet mogelijk bij correcte exponent-bepaling):
/// retourneer 10 als fallback-ondergrens.
fn round_down_tabel_x1(basis_f: f64) -> u32 {
    // Vind grootste v <= floor(basis_f). We vergelijken hier met de float
    // direct — dat vermijdt een `f64 as u32` cast-lint en is semantisch gelijk.
    let mut result = TABEL_X1_BASIS[0];
    for &v in &TABEL_X1_BASIS {
        if f64::from(v) <= basis_f {
            result = v;
        } else {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Voorbeelden uit norm (naar boven) ---

    #[test]
    fn voorbeeld_0_33_naar_boven() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(0.33, r) - 0.34).abs() < 1e-12);
    }

    #[test]
    fn voorbeeld_3_3_naar_boven() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(3.3, r) - 3.4).abs() < 1e-12);
    }

    #[test]
    fn voorbeeld_33_naar_boven() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(33.0, r) - 34.0).abs() < 1e-12);
    }

    #[test]
    fn voorbeeld_332_naar_boven() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(332.0, r) - 340.0).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_3327_naar_boven() {
        let r = RoundingRule::StandByVerlies;
        assert!((round_to_significant_figures(3327.0, r) - 3400.0).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_3450_naar_boven() {
        let r = RoundingRule::ParasitairVermogenVerlichting;
        assert!((round_to_significant_figures(3450.0, r) - 3600.0).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_3400_naar_boven_gaat_naar_3600() {
        // §X.2: "eerstvolgende hogere getal" is strict greater — ook bij
        // exacte match gaat de waarde een stap omhoog.
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(3400.0, r) - 3600.0).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_0_09_naar_boven_gaat_naar_0_095() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(0.09, r) - 0.095).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_822_naar_boven() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(822.0, r) - 850.0).abs() < 1e-9);
    }

    // --- Voorbeelden uit norm (naar beneden) ---

    #[test]
    fn voorbeeld_0_33_naar_beneden() {
        let d = RoundingRule::HernieuwbareEnergieBijdrage;
        assert!((round_to_significant_figures(0.33, d) - 0.32).abs() < 1e-12);
    }

    #[test]
    fn voorbeeld_3_3_naar_beneden() {
        let d = RoundingRule::BwpWarmteverliesCompensatie;
        assert!((round_to_significant_figures(3.3, d) - 3.2).abs() < 1e-12);
    }

    #[test]
    fn voorbeeld_3327_naar_beneden() {
        let d = RoundingRule::HernieuwbareEnergieBijdrage;
        assert!((round_to_significant_figures(3327.0, d) - 3200.0).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_3450_naar_beneden() {
        let d = RoundingRule::ZonneboilerKlasse;
        assert!((round_to_significant_figures(3450.0, d) - 3400.0).abs() < 1e-9);
    }

    #[test]
    fn voorbeeld_3400_blijft_staan_naar_beneden() {
        // 3400 is exact in tabel X.1 als patroon 34 → blijft staan.
        let d = RoundingRule::ZonneboilerKlasse;
        assert!((round_to_significant_figures(3400.0, d) - 3400.0).abs() < 1e-9);
    }

    // --- Randgevallen ---

    #[test]
    #[allow(clippy::float_cmp)] // 0.0 moet exact 0.0 blijven (§X.2)
    fn nul_wordt_nooit_afgerond() {
        for rule in [
            RoundingRule::VermogenInstallatiecomponent,
            RoundingRule::HernieuwbareEnergieBijdrage,
            RoundingRule::StandByVerlies,
            RoundingRule::BwpWarmteverliesCompensatie,
            RoundingRule::ZonneboilerKlasse,
            RoundingRule::ParasitairVermogenVerlichting,
        ] {
            assert_eq!(round_to_significant_figures(0.0, rule), 0.0);
        }
    }

    #[test]
    #[allow(clippy::float_cmp)] // ±∞ moet bit-identiek blijven
    fn niet_eindige_waarden_blijven_staan() {
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!(round_to_significant_figures(f64::NAN, r).is_nan());
        assert_eq!(
            round_to_significant_figures(f64::INFINITY, r),
            f64::INFINITY
        );
        assert_eq!(
            round_to_significant_figures(f64::NEG_INFINITY, r),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn teken_wordt_behouden() {
        // De norm noemt geen negatieve waarden, maar voor robustness: teken behouden.
        let r = RoundingRule::VermogenInstallatiecomponent;
        assert!((round_to_significant_figures(-33.0, r) - -34.0).abs() < 1e-12);
    }

    #[test]
    fn tabel_x1_basis_is_strikt_stijgend() {
        for pair in TABEL_X1_BASIS.windows(2) {
            assert!(pair[0] < pair[1], "Niet strikt stijgend: {pair:?}");
        }
    }

    #[test]
    fn tabel_x1_basis_heeft_33_waarden() {
        // Conform bijlage X tabel X.1.
        assert_eq!(TABEL_X1_BASIS.len(), 33);
    }
}
