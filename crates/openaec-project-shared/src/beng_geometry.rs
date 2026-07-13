//! Additief `beng_geometry`-invoerblok op [`crate::ProjectV2`] — een
//! **gevel-georiënteerde** geometrie-laag die 1-op-1 op het Uniec 3 /
//! NTA 8800-invoermodel zit (F6).
//!
//! ## Waarom een tweede geometrie-laag
//!
//! De bestaande [`crate::geometry::SharedGeometry`] is **ruimte-georiënteerd**
//! (`Space` → `Construction` → `Opening`) en wordt in de studio gevuld met
//! **binnen-oppervlakten** uit de ISSO 51-warmteverliesmodeller. NTA 8800/Uniec
//! rekent daarentegen met **buiten-oppervlakten per gevel op rekenzone-niveau**;
//! die mismatch is de bewezen hoofdverdachte van de Q_H;nd-onderschatting
//! (BENG 1 −26 % op de Aalten-case). Zie
//! `docs/2026-07-12-uniec-velden-inventarisatie.md` §4b/§6.
//!
//! Dit blok is daarom **additief**: het hangt als
//! `Option<BengGeometry>` op [`crate::ProjectV2`] náást `SharedGeometry`, zodat
//! de ISSO 51/53-tak volledig ongemoeid blijft. Bestaande project-JSON's zonder
//! `beng_geometry`-veld deserialiseren ongewijzigd naar `None`.
//!
//! ## Structuur — spiegelt Uniecs boomstructuur
//!
//! ```text
//! BengGeometry
//! ├── opaque_defs: OpaqueConstructionDef[]   bouwkundige bibliotheek (Rc/U per code)
//! ├── window_defs: WindowDef[]               kozijnmerken (U/ggl/opp per merk)
//! └── zones: BengZone[]                      rekenzone (A_g + bouwwijze)
//!     └── gevels: BengBoundary[]             begrenzing = de thermische schil
//!         └── ramen: BengWindowPlacement[]   kozijn-ref + aantal + belemmering/zonwering
//! ```
//!
//! De twee bibliotheken (`opaque_defs`, `window_defs`) leven op
//! `BengGeometry`-niveau en worden door de gevels/ramen via id gerefereerd —
//! precies Uniecs tweelaags model (definitie ↔ plaatsing). Dat maakt hergebruik
//! van een constructie/kozijnmerk over meerdere gevels mogelijk.
//!
//! ## Uniec-veldnamen in de doc-comments
//!
//! Elk veld draagt in de doc-comment de bijbehorende Uniec-interne veldcode
//! (bv. `BEGR_GEVEL`, `CONSTRD_LIB`, `CONSTRT_BESCH`) en/of NTA 8800-referentie,
//! zodat de invoer herleidbaar is naar de certified-tool. De codes komen uit de
//! velden-inventarisatie (§2 referentie-enums).
//!
//! ## Scope F6 fase 1 — alleen data-laag
//!
//! Dit blok wordt in fase 1 **nog niet** door [`crate::compute_beng`] gelezen;
//! het is puur invoer-DTO + validatie. De vertaling naar de rekenzone-geometrie
//! van de service-crates (de F2b-orchestrator-brug) is fase 2.

use nta8800_model::{
    location::Orientation, MovableSunShading, ModelError, ModelResult, Obstruction,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------

/// Gevel-georiënteerde BENG-geometrie-invoer (F6). Zie module-doc.
///
/// Additief invoerblok náást [`crate::geometry::SharedGeometry`]; alle velden
/// hebben een default zodat een half-ingevuld formulier geldig blijft en
/// bestaande JSON byte-identiek round-trippt.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BengGeometry {
    /// Bouwkundige bibliotheek — opake constructie-definities (Uniec
    /// `LIBCONSTRD_*`). Door de gevels gerefereerd via
    /// [`BengBoundary::constructie_ref`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opaque_defs: Vec<OpaqueConstructionDef>,

    /// Kozijn-bibliotheek — kozijnmerk-definities (Uniec `LIBCONSTRT_*`). Door
    /// de ramen gerefereerd via [`BengWindowPlacement::kozijn_ref`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub window_defs: Vec<WindowDef>,

    /// Rekenzones (Uniec `RZ`). Meestal één voor een grondgebonden woning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zones: Vec<BengZone>,
}

impl BengGeometry {
    /// Valideer referentie-integriteit + plausibiliteit van het hele blok.
    ///
    /// Volgt het `nta8800-model`-patroon: geeft een [`ModelError`] terug in
    /// plaats van te panieken (idem [`nta8800_model::resolve_zone`]). Gecheckt:
    ///
    /// - **Uniekheid:** `zones[].id` uniek; `gevels[].id` **globaal** uniek over
    ///   alle zones heen (relevant bij multi-zone); `opaque_defs[].id` en
    ///   `window_defs[].id` elk uniek.
    /// - **Referenties:** elke [`BengBoundary::constructie_ref`] wijst naar een
    ///   bestaande [`OpaqueConstructionDef`]; elke
    ///   [`BengWindowPlacement::kozijn_ref`] naar een bestaande [`WindowDef`]
    ///   → anders [`ModelError::ReferenceNotFound`].
    /// - **Plausibiliteit:** `a_g_m2 > 0`, `bruto_buiten_opp_m2 > 0`,
    ///   `aantal >= 1`, `helling_deg ∈ 0..=180` (indien aanwezig), de
    ///   `omtrek_p_m`-verplichting bij vloer-op-grond (P/A-methode), en dat het
    ///   totale raamoppervlak per gevel het bruto gevelvlak niet overschrijdt.
    ///
    /// **Leeg is geldig.** Een blok zonder `zones` (of een zone zonder `gevels`,
    /// of een gevel zonder `ramen`) is bewust `Ok`: dit is een invoer-DTO voor
    /// een mogelijk half-ingevuld formulier, geen eis dat de schil compleet is.
    /// De validatie bewaakt alleen dat wát er staat consistent is.
    ///
    /// # Errors
    ///
    /// De eerste overtreding als [`ModelError`]
    /// ([`InvalidInput`](ModelError::InvalidInput) /
    /// [`OutOfRange`](ModelError::OutOfRange) /
    /// [`ReferenceNotFound`](ModelError::ReferenceNotFound)).
    pub fn validate(&self) -> ModelResult<()> {
        check_unique(self.opaque_defs.iter().map(|d| d.id.as_str()), "OpaqueConstructionDef")?;
        check_unique(self.window_defs.iter().map(|d| d.id.as_str()), "WindowDef")?;
        check_unique(self.zones.iter().map(|z| z.id.as_str()), "BengZone")?;
        // Gevel-id's zijn **globaal** uniek over alle zones heen (niet enkel per
        // zone): bij multi-zone utiliteit refereren rapportage en koudebrug-
        // koppelingen aan een begrenzingsvlak op id, dus een dubbele id in twee
        // zones zou ambigu zijn.
        check_unique(
            self.zones.iter().flat_map(|z| z.gevels.iter()).map(|g| g.id.as_str()),
            "BengBoundary",
        )?;

        for def in &self.opaque_defs {
            def.validate()?;
        }
        for def in &self.window_defs {
            def.validate()?;
        }
        for zone in &self.zones {
            zone.validate(self)?;
        }
        Ok(())
    }

    /// Zoek een opake constructie-definitie op id. `None` als afwezig.
    #[must_use]
    pub fn opaque_def(&self, id: &str) -> Option<&OpaqueConstructionDef> {
        self.opaque_defs.iter().find(|d| d.id == id)
    }

    /// Zoek een kozijnmerk-definitie op id. `None` als afwezig.
    #[must_use]
    pub fn window_def(&self, id: &str) -> Option<&WindowDef> {
        self.window_defs.iter().find(|d| d.id == id)
    }
}

// ---------------------------------------------------------------------------
// Bouwkundige bibliotheek — opake constructies
// ---------------------------------------------------------------------------

/// Opake constructie-definitie in de bouwkundige bibliotheek (Uniec
/// `LIBCONSTRD_*`, methode `VRIJE_INV` = vrije invoer).
///
/// Wordt door één of meer [`BengBoundary`]s gerefereerd; dit is de
/// definitie-kant van Uniecs tweelaags model (definitie ↔ plaatsing).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OpaqueConstructionDef {
    /// Unieke id binnen [`BengGeometry::opaque_defs`] (referentiedoel van
    /// [`BengBoundary::constructie_ref`]).
    pub id: String,

    /// Mens-leesbare omschrijving (Uniec `LIBCONSTRD_OMSCHR`, bv. "Wand").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub omschrijving: String,

    /// Vlak-type (Uniec `LIBCONSTRD_TYPE`).
    pub kind: VlakType,

    /// Thermische kwaliteit: Rc (opake vrije invoer, m²·K/W) of een direct
    /// gegeven U (W/(m²·K)). Uniec hanteert hier Rc; het model laat beide toe.
    pub thermal: RcOrU,
}

impl OpaqueConstructionDef {
    fn validate(&self) -> ModelResult<()> {
        self.thermal.validate(&format!("OpaqueConstructionDef[{}].thermal", self.id))
    }
}

/// Thermische kwaliteit van een opake constructie — Rc of U.
///
/// Uniec voert opake constructies in als **Rc** (warmteweerstand van de
/// constructie zónder oppervlakteweerstanden); een direct gegeven **U** is de
/// alternatieve invoer. De F2b-orchestrator zet Rc via de
/// oppervlakteweerstanden (`R_si`/`R_se`, afhankelijk van vlak-type) om naar U.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RcOrU {
    /// Warmteweerstand Rc in m²·K/W (Uniec `LIBCONSTRD_RC`, vrije invoer).
    Rc(f64),
    /// Warmtedoorgangscoëfficiënt U in W/(m²·K).
    U(f64),
}

impl RcOrU {
    fn validate(&self, context: &str) -> ModelResult<()> {
        let (val, name) = match self {
            RcOrU::Rc(v) => (*v, "Rc"),
            RcOrU::U(v) => (*v, "U"),
        };
        if !val.is_finite() || val <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("{context} ({name})"),
                reason: "moet > 0 en eindig zijn".into(),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Kozijn-bibliotheek — kozijnmerken
// ---------------------------------------------------------------------------

/// Kozijnmerk-definitie in de kozijn-bibliotheek (Uniec `LIBCONSTRT_*`, modus
/// `oppervlakte per kozijnmerk invoeren`).
///
/// [`Self::area_m2`] is de **oppervlakte per exemplaar**; een plaatsing
/// ([`BengWindowPlacement`]) vermenigvuldigt die met `aantal`. Zo geeft
/// bibliotheek-merk "C" (0,36 m²) met `aantal = 2` op een gevel 0,72 m².
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WindowDef {
    /// Unieke id binnen [`BengGeometry::window_defs`] (referentiedoel van
    /// [`BengWindowPlacement::kozijn_ref`]).
    pub id: String,

    /// Kozijnmerk-omschrijving (Uniec `LIBCONSTRT_OMSCHR`, bv. "A", "dakraam").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub omschrijving: String,

    /// Type transparante/opake vulling (Uniec `LIBCONSTRT_TYPE`).
    pub kind: KozijnType,

    /// Samengestelde U-waarde in W/(m²·K) (glas + kozijn, Uniec `U`).
    pub u_w_per_m2k: f64,

    /// Zonnewarmtedoorlatingsfactor g (0..=1, Uniec `ggl`). `None`/0 voor een
    /// opake deur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ggl: Option<f64>,

    /// Oppervlakte **per exemplaar** in m² (Uniec kozijnmerk-`oppervlakte`). De
    /// totale bijdrage aan een gevel is `area_m2 · aantal` van de plaatsing.
    pub area_m2: f64,
}

impl WindowDef {
    fn validate(&self) -> ModelResult<()> {
        if !self.u_w_per_m2k.is_finite() || self.u_w_per_m2k <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("WindowDef[{}].u_w_per_m2k", self.id),
                reason: "moet > 0 en eindig zijn".into(),
            });
        }
        if let Some(g) = self.ggl {
            if !g.is_finite() || !(0.0..=1.0).contains(&g) {
                return Err(ModelError::OutOfRange {
                    field: format!("WindowDef[{}].ggl", self.id),
                    range: "0.0..=1.0".into(),
                    value: format!("{g}"),
                });
            }
        }
        if !self.area_m2.is_finite() || self.area_m2 <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("WindowDef[{}].area_m2", self.id),
                reason: "moet > 0 en eindig zijn".into(),
            });
        }
        Ok(())
    }
}

/// Type kozijn-vulling (Uniec `LIBCONSTRT_TYPE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KozijnType {
    /// Raam (transparant, draagt g-waarde). Uniec `TRANSTYPE_RAAM`.
    Raam,
    /// Deur (opaak of glas). Uniec `TRANSTYPE_DEUR`.
    Deur,
    /// Paneel in kozijn (opaak vlak binnen een kozijn). Uniec `TRANSTYPE_PANEEL`.
    PaneelInKozijn,
}

// ---------------------------------------------------------------------------
// Rekenzone
// ---------------------------------------------------------------------------

/// Rekenzone (Uniec `RZ`, NTA 8800 §6.2) — thermisch samenhangend gebied met
/// zijn eigen begrenzing (de thermische schil).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BengZone {
    /// Unieke id binnen [`BengGeometry::zones`].
    pub id: String,

    /// Rekenzone-omschrijving (Uniec `RZ_OMSCHR`, bv. "woning").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub naam: String,

    /// Gebruiksoppervlakte A_g in m² (Uniec `A_g gebruiksoppervlak`, NTA 8800
    /// §6). Noemer van de BENG-indicatoren.
    pub a_g_m2: f64,

    /// Bouwwijze vloer (thermische massa, Uniec `RZ_BOUWW_VL`, bv.
    /// `CONSTRM_FL_26` = massief beton zeer zwaar). Vrij-veld met de Uniec-code;
    /// relevant voor TOjuli/dynamica, nog niet verrekend in fase 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bouwwijze_vloer: Option<String>,

    /// Bouwwijze wand (Uniec `RZ_BOUWW_W`, bv. `CONSTRM_W_11` = hsb licht).
    /// Zie [`Self::bouwwijze_vloer`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bouwwijze_wand: Option<String>,

    /// Woningtype (Uniec `UNIT_TYPEWON`, bv. `TWON_VRIJ_K` = vrijstaand met
    /// kap). Bepaalt onder meer het infiltratie-forfait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub woningtype: Option<String>,

    /// De begrenzingsvlakken die de thermische schil van deze zone vormen
    /// (Uniec `Begrenzing`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gevels: Vec<BengBoundary>,
}

impl BengZone {
    fn validate(&self, geo: &BengGeometry) -> ModelResult<()> {
        if !self.a_g_m2.is_finite() || self.a_g_m2 <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("BengZone[{}].a_g_m2", self.id),
                reason: "moet > 0 en eindig zijn".into(),
            });
        }
        // Gevel-id-uniekheid wordt globaal (over alle zones) getoetst in
        // [`BengGeometry::validate`]; hier alleen de per-gevel-validatie.
        for gevel in &self.gevels {
            gevel.validate(geo)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Begrenzing (gevel)
// ---------------------------------------------------------------------------

/// Eén begrenzingsvlak van de thermische schil (Uniec `Begrenzing` +
/// `Constructies/{vlak}`) — de gevel-georiënteerde kern van dit blok.
///
/// Combineert de begrenzing (vlak-type, grenst-aan, buiten-oppervlak, helling,
/// omtrek) met de constructie-plaatsing (opake-ref + ramen). Uniec splitst dit
/// over twee pagina's (`Begrenzing` en `Constructies/{id}`); wij houden het per
/// vlak bij elkaar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BengBoundary {
    /// Unieke id binnen de zone (Uniec begrenzingsvlak-id).
    pub id: String,

    /// Vlak-omschrijving (Uniec `BEGR_OMSCHR`, bv. "Wand", "Dak").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub omschrijving: String,

    /// Vlak-type (Uniec `BEGR_VLAK`).
    pub vlak_type: VlakType,

    /// Waar het vlak aan grenst (Uniec `BEGR_VLOER`/`BEGR_GEVEL`/`BEGR_DAK`);
    /// voor buitenlucht/AOS draagt de variant de oriëntatie mee, zie
    /// [`BengAdjacency`].
    pub grenst_aan: BengAdjacency,

    /// Bruto **buiten**-oppervlak in m² (Uniec begrenzing-`opp`). Dit is de
    /// kern-heroriëntatie: buiten-oppervlak per gevel, niet binnen-oppervlak per
    /// kamer.
    pub bruto_buiten_opp_m2: f64,

    /// Helling in graden t.o.v. horizontaal (Uniec `BEGR_HEL`; 90 = gevel,
    /// 15 = hellend dak). `None` voor een vloer ("n.v.t." in Uniec).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub helling_deg: Option<f64>,

    /// Omtrek P van het vloerveld in m (Uniec `omtrek van het vloerveld (P)`).
    /// Verplicht bij een vloer-op-grond (P/A-methode, NTA 8800 bijlage);
    /// afwezig voor overige vlakken. Zie [`BengAdjacency::requires_omtrek`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omtrek_p_m: Option<f64>,

    /// Referentie naar de opake constructie in [`BengGeometry::opaque_defs`]
    /// (Uniec `CONSTRD_LIB`).
    pub constructie_ref: String,

    /// Ramen/deuren op dit vlak (Uniec `Constructies/{vlak}` → kozijn-plaatsingen).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ramen: Vec<BengWindowPlacement>,
}

impl BengBoundary {
    fn validate(&self, geo: &BengGeometry) -> ModelResult<()> {
        if !self.bruto_buiten_opp_m2.is_finite() || self.bruto_buiten_opp_m2 <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("BengBoundary[{}].bruto_buiten_opp_m2", self.id),
                reason: "moet > 0 en eindig zijn".into(),
            });
        }
        if let Some(h) = self.helling_deg {
            if !h.is_finite() || !(0.0..=180.0).contains(&h) {
                return Err(ModelError::OutOfRange {
                    field: format!("BengBoundary[{}].helling_deg", self.id),
                    range: "0.0..=180.0".into(),
                    value: format!("{h}"),
                });
            }
        }
        // Omtrek P is verplicht bij vloer-op-grond (P/A-methode) en moet dan > 0
        // zijn; bij overige vlakken is een aanwezige P onschadelijk maar moet,
        // indien gegeven, eveneens > 0 zijn.
        match self.omtrek_p_m {
            Some(p) if !p.is_finite() || p <= 0.0 => {
                return Err(ModelError::InvalidInput {
                    context: format!("BengBoundary[{}].omtrek_p_m", self.id),
                    reason: "moet > 0 en eindig zijn".into(),
                });
            }
            None if self.grenst_aan.requires_omtrek() => {
                return Err(ModelError::InvalidInput {
                    context: format!("BengBoundary[{}].omtrek_p_m", self.id),
                    reason: "verplicht bij vloer-op-grond (P/A-methode)".into(),
                });
            }
            _ => {}
        }
        if geo.opaque_def(&self.constructie_ref).is_none() {
            return Err(ModelError::ReferenceNotFound {
                kind: "OpaqueConstructionDef".into(),
                id: self.constructie_ref.clone(),
            });
        }
        for raam in &self.ramen {
            raam.validate(geo)?;
        }
        // Het totale kozijn-oppervlak (Σ aantal · WindowDef::area_m2) kan niet
        // groter zijn dan het bruto gevelvlak — de ramen zitten ín de gevel. De
        // raam-refs resolven hierboven al, dus een ontbrekende ref telt als 0 en
        // is niet de fout die deze check rapporteert. Gelijkheid is toegestaan:
        // een volledig beglaasde pui (opaak = 0) is een geldig randgeval.
        let ramen_opp: f64 = self
            .ramen
            .iter()
            .map(|r| {
                geo.window_def(&r.kozijn_ref)
                    .map_or(0.0, |d| f64::from(r.aantal) * d.area_m2)
            })
            .sum();
        if ramen_opp - self.bruto_buiten_opp_m2 > 1e-9 {
            return Err(ModelError::InvalidInput {
                context: format!("BengBoundary[{}].ramen", self.id),
                reason: format!(
                    "totaal raamoppervlak {ramen_opp:.4} m² overschrijdt bruto gevelvlak {:.4} m²",
                    self.bruto_buiten_opp_m2
                ),
            });
        }
        Ok(())
    }
}

/// Vlak-type van een begrenzing of opake constructie-definitie (Uniec
/// `BEGR_VLAK` / `LIBCONSTRD_TYPE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VlakType {
    /// Vloer. Uniec `VLAK_VLOER` / `LIBVLAK_VLOER`.
    Vloer,
    /// Vloer boven buitenlucht. Uniec `vloer boven buitenlucht`.
    VloerBovenBuitenlucht,
    /// Gevel (verticale wand). Uniec `VLAK_GEVEL` / `LIBVLAK_GEVEL`.
    Gevel,
    /// Dak. Uniec `VLAK_DAK` / `LIBVLAK_DAK`.
    Dak,
    /// Kelderwand. Uniec `kelderwand`.
    Kelderwand,
    /// Bodem (alleen bibliotheek-type). Uniec `bodem`.
    Bodem,
}

/// Waar een begrenzingsvlak aan grenst (Uniec `BEGR_VLOER`/`BEGR_GEVEL`/
/// `BEGR_DAK` referentie-enums, §2 van de velden-inventarisatie).
///
/// Deze typologie is preciezer dan [`crate::geometry::BoundaryKind`]: ze dekt de
/// vloer-subtypes (op/onder maaiveld × kruipruimte/grond/kelder), de
/// forfaitaire onverwarmde-buffer-begrenzingen (AOS/AOR), sterk geventileerd en
/// water. Voor buitenlucht draagt de variant de **8-punts oriëntatie** mee
/// ([`Orientation::Horizontaal`] = Uniecs `HOR` voor een plat dak) — dat is hoe
/// Uniec de oriëntatie codeert (`GVL_BTNL_N` etc.), dus één bron van waarheid.
/// Bij [`Self::AosForfaitair`] is de oriëntatie **optioneel**: Uniec kent zowel
/// een gevel/dak-AOS *mét* richting als een vloer-AOS *zónder* richting (§2). We
/// modelleren dat als één variant met `Option<Orientation>` (i.p.v. twee
/// aparte varianten), zodat de JSON compact blijft — `None` serialiseert als
/// `{"aos_forfaitair":{}}`. Gebruik [`Self::orientatie`] om de richting te lezen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BengAdjacency {
    /// Op/boven maaiveld, boven kruipruimte. Uniec `BEGR_VLOER`-optie.
    VloerOpMaaiveldBovenKruipruimte,
    /// Op/boven maaiveld, boven grond/spouw (z ≤ 0,3). Uniec `VL_MV_GRSP`.
    VloerOpMaaiveldBovenGrond,
    /// Op/boven maaiveld, boven onverwarmde kelder. Uniec `BEGR_VLOER`-optie.
    VloerOpMaaiveldBovenOnverwarmdeKelder,
    /// Onder maaiveld, boven kruipruimte. Uniec `BEGR_VLOER`-optie.
    VloerOnderMaaiveldBovenKruipruimte,
    /// Onder maaiveld, boven grond/spouw (z ≤ 0,3). Uniec `BEGR_VLOER`-optie.
    VloerOnderMaaiveldBovenGrond,
    /// Onder maaiveld, boven onverwarmde kelder. Uniec `BEGR_VLOER`-optie.
    VloerOnderMaaiveldBovenOnverwarmdeKelder,
    /// Buitenlucht met oriëntatie. Uniec `GVL_BTNL_{Z..NW}` / `DAK_BTNL_*`
    /// (`HOR` = [`Orientation::Horizontaal`]).
    Buitenlucht {
        /// Kompasrichting (of `Horizontaal` voor een plat dak).
        orientatie: Orientation,
    },
    /// Sterk geventileerde ruimte/spouw. Uniec `sterk geventileerd`.
    SterkGeventileerd,
    /// Open water. Uniec `water`.
    Water,
    /// Aangrenzende verwarmde ruimte (AVR). Uniec `AVR`.
    AangrenzendeVerwarmdeRuimte,
    /// Aangrenzende onverwarmde serre/ruimte (AOS), forfaitair. Uniec
    /// `AOS forfaitair; {richting}` (gevel/dak, mét richting) óf `AOS forfaitair`
    /// (vloer, zónder richting).
    AosForfaitair {
        /// Kompasrichting (of `Horizontaal`) van het AOS-vlak; `None` voor de
        /// vloer-AOS die geen oriëntatie draagt.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        orientatie: Option<Orientation>,
    },
    /// Aangrenzende onverwarmde ruimte (AOR), forfaitair. Uniec `AOR forfaitair`.
    AorForfaitair,
}

impl BengAdjacency {
    /// De oriëntatie die in de variant zit (buitenlucht/AOS), of `None` voor de
    /// oriëntatieloze begrenzingen (vloer-subtypes, water, AVR, AOR, sterk
    /// geventileerd).
    #[must_use]
    pub fn orientatie(&self) -> Option<Orientation> {
        match self {
            BengAdjacency::Buitenlucht { orientatie } => Some(*orientatie),
            BengAdjacency::AosForfaitair { orientatie } => *orientatie,
            _ => None,
        }
    }

    /// `true` voor de vloer-op-grond-begrenzingen die de P/A-methode gebruiken
    /// en dus een omtrek P vereisen (grond/spouw-contact, z ≤ 0,3). De
    /// buffer-varianten (kruipruimte/kelder) lopen via een temperatuurcorrectie
    /// en vereisen geen P.
    #[must_use]
    pub fn requires_omtrek(&self) -> bool {
        matches!(
            self,
            BengAdjacency::VloerOpMaaiveldBovenGrond | BengAdjacency::VloerOnderMaaiveldBovenGrond
        )
    }
}

// ---------------------------------------------------------------------------
// Kozijn-plaatsing
// ---------------------------------------------------------------------------

/// Een kozijn-plaatsing op een gevel (Uniec `Constructies/{vlak}` →
/// `CONSTRT_LIB` + belemmering/zonwering/zomernachtventilatie).
///
/// Verwijst naar een [`WindowDef`] en geeft het aantal exemplaren plus de
/// per-raam-eigenschappen. Belemmering en zonwering hergebruiken de bestaande
/// `nta8800-model`-typen (die al op [`crate::geometry::Opening`] hangen) — geen
/// duplicaat-typen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BengWindowPlacement {
    /// Referentie naar het kozijnmerk in [`BengGeometry::window_defs`] (Uniec
    /// `CONSTRT_LIB`).
    pub kozijn_ref: String,

    /// Aantal identieke exemplaren op deze gevel (Uniec plaatsings-`aantal`).
    /// De totale oppervlakte is `aantal · WindowDef::area_m2`.
    #[serde(default = "default_aantal")]
    pub aantal: u32,

    /// Externe belemmering (Uniec `CONSTRT_BESCH`, NTA 8800 §17.3). Default
    /// [`Obstruction::None`]; bij de default niet geserialiseerd zodat JSON
    /// byte-identiek blijft.
    ///
    /// Let op: `nta8800-model::Obstruction` modelleert in V1 alleen
    /// `None`/`Minimal`; Uniecs rijkere belemmeringsvarianten (zijbelemmering,
    /// overstek met hoogtehoek) zijn V2 en worden voorlopig op `Minimal`
    /// benaderd.
    #[serde(default, skip_serializing_if = "Obstruction::is_none")]
    pub belemmering: Obstruction,

    /// Beweegbare zonwering (Uniec `CONSTRT_ZONW`, NTA 8800 §7.6.6.1.4).
    /// `None` = geen zonwering (Uniec `ZONW_GEEN`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zonwering: Option<MovableSunShading>,

    /// Zomernachtventilatie via dit raam aanwezig (Uniec `CONSTRT_ZNVENT`;
    /// `ZOMERNVENT_NAANW` = niet aanwezig = `false`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub zomernachtventilatie: bool,
}

impl BengWindowPlacement {
    fn validate(&self, geo: &BengGeometry) -> ModelResult<()> {
        if self.aantal == 0 {
            return Err(ModelError::InvalidInput {
                context: format!("BengWindowPlacement[{}].aantal", self.kozijn_ref),
                reason: "moet >= 1 zijn".into(),
            });
        }
        if geo.window_def(&self.kozijn_ref).is_none() {
            return Err(ModelError::ReferenceNotFound {
                kind: "WindowDef".into(),
                id: self.kozijn_ref.clone(),
            });
        }
        Ok(())
    }
}

fn default_aantal() -> u32 {
    1
}

// ---------------------------------------------------------------------------
// Interne helpers
// ---------------------------------------------------------------------------

/// Controleer dat een id-reeks geen duplicaten bevat.
///
/// Gebruikt [`ModelError::InvalidInput`] met de duplicaat-id in de reden, zodat
/// de fout aansluit op het bestaande foutmodel zonder een nieuwe variant.
fn check_unique<'a>(ids: impl Iterator<Item = &'a str>, kind: &str) -> ModelResult<()> {
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(ModelError::InvalidInput {
                context: format!("{kind}.id"),
                reason: format!("dubbele id '{id}'"),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::geometry::ShadingControl;

    fn minimal_geo() -> BengGeometry {
        BengGeometry {
            opaque_defs: vec![OpaqueConstructionDef {
                id: "def-wand".into(),
                omschrijving: "Wand".into(),
                kind: VlakType::Gevel,
                thermal: RcOrU::Rc(4.70),
            }],
            window_defs: vec![WindowDef {
                id: "merk-a".into(),
                omschrijving: "A".into(),
                kind: KozijnType::Raam,
                u_w_per_m2k: 1.3,
                ggl: Some(0.40),
                area_m2: 4.12,
            }],
            zones: vec![BengZone {
                id: "rz-woning".into(),
                naam: "woning".into(),
                a_g_m2: 67.0,
                bouwwijze_vloer: None,
                bouwwijze_wand: None,
                woningtype: None,
                gevels: vec![BengBoundary {
                    id: "gevel-o".into(),
                    omschrijving: "Wand".into(),
                    vlak_type: VlakType::Gevel,
                    grenst_aan: BengAdjacency::Buitenlucht {
                        orientatie: Orientation::Oost,
                    },
                    bruto_buiten_opp_m2: 23.81,
                    helling_deg: Some(90.0),
                    omtrek_p_m: None,
                    constructie_ref: "def-wand".into(),
                    ramen: vec![BengWindowPlacement {
                        kozijn_ref: "merk-a".into(),
                        aantal: 1,
                        belemmering: Obstruction::Minimal,
                        zonwering: None,
                        zomernachtventilatie: false,
                    }],
                }],
            }],
        }
    }

    #[test]
    fn empty_geometry_round_trips_to_compact_json() {
        let g = BengGeometry::default();
        let json = serde_json::to_string(&g).unwrap();
        assert_eq!(json, "{}");
        let back: BengGeometry = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn filled_geometry_round_trips() {
        let g = minimal_geo();
        let json = serde_json::to_string(&g).unwrap();
        let back: BengGeometry = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }

    #[test]
    fn minimal_geometry_validates() {
        assert!(minimal_geo().validate().is_ok());
    }

    #[test]
    fn unknown_constructie_ref_is_reference_not_found() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].constructie_ref = "ontbreekt".into();
        match g.validate().unwrap_err() {
            ModelError::ReferenceNotFound { kind, id } => {
                assert_eq!(kind, "OpaqueConstructionDef");
                assert_eq!(id, "ontbreekt");
            }
            other => panic!("verwacht ReferenceNotFound, kreeg {other:?}"),
        }
    }

    #[test]
    fn unknown_kozijn_ref_is_reference_not_found() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].ramen[0].kozijn_ref = "spook".into();
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::ReferenceNotFound { kind, .. } if kind == "WindowDef"
        ));
    }

    #[test]
    fn duplicate_zone_id_is_invalid_input() {
        let mut g = minimal_geo();
        let dup = g.zones[0].clone();
        g.zones.push(dup);
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { .. }
        ));
    }

    #[test]
    fn duplicate_gevel_id_within_zone_is_invalid_input() {
        let mut g = minimal_geo();
        let dup = g.zones[0].gevels[0].clone();
        g.zones[0].gevels.push(dup);
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { context, .. } if context == "BengBoundary.id"
        ));
    }

    #[test]
    fn duplicate_gevel_id_across_zones_is_invalid_input() {
        // Twee zones met elk een gevel die dezelfde id draagt → globaal dubbel.
        let mut g = minimal_geo();
        let mut zone2 = g.zones[0].clone();
        zone2.id = "rz-2".into(); // zone-id zelf uniek
        // gevel-id ("gevel-o") blijft gelijk aan die in zone 1 → conflict.
        g.zones.push(zone2);
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { context, reason }
                if context == "BengBoundary.id" && reason.contains("gevel-o")
        ));
    }

    #[test]
    fn distinct_gevel_ids_across_zones_are_ok() {
        let mut g = minimal_geo();
        let mut zone2 = g.zones[0].clone();
        zone2.id = "rz-2".into();
        zone2.gevels[0].id = "gevel-o2".into(); // globaal uniek
        g.zones.push(zone2);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn floor_on_ground_requires_omtrek() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].vlak_type = VlakType::Vloer;
        g.zones[0].gevels[0].grenst_aan = BengAdjacency::VloerOpMaaiveldBovenGrond;
        g.zones[0].gevels[0].helling_deg = None;
        g.zones[0].gevels[0].omtrek_p_m = None;
        g.zones[0].gevels[0].ramen.clear();
        // Zonder omtrek P → InvalidInput.
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { context, .. } if context.contains("omtrek_p_m")
        ));
        // Mét omtrek P → geldig.
        g.zones[0].gevels[0].omtrek_p_m = Some(32.92);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn non_positive_area_is_invalid_input() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].bruto_buiten_opp_m2 = 0.0;
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { .. }
        ));
    }

    #[test]
    fn helling_out_of_range_is_out_of_range() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].helling_deg = Some(200.0);
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::OutOfRange { .. }
        ));
    }

    #[test]
    fn zero_aantal_is_invalid_input() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].ramen[0].aantal = 0;
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { .. }
        ));
    }

    #[test]
    fn ramen_area_exceeding_gevel_is_invalid_input() {
        let mut g = minimal_geo();
        // merk-a = 4,12 m²; 10 exemplaren = 41,2 m² > bruto 23,81 m².
        g.zones[0].gevels[0].ramen[0].aantal = 10;
        assert!(matches!(
            g.validate().unwrap_err(),
            ModelError::InvalidInput { context, .. } if context.ends_with(".ramen")
        ));
    }

    #[test]
    fn ramen_area_exactly_equal_to_gevel_is_ok() {
        // Volledig beglaasde pui / deur-in-kozijn-gevel: opaak = 0 is geldig.
        let mut g = minimal_geo();
        g.zones[0].gevels[0].bruto_buiten_opp_m2 = 4.12; // == merk-a, aantal 1
        assert!(g.validate().is_ok());
    }

    #[test]
    fn aos_forfaitair_vloer_has_no_orientation() {
        // Vloer-AOS zonder richting → compacte JSON, orientatie() = None.
        let a = BengAdjacency::AosForfaitair { orientatie: None };
        assert_eq!(a.orientatie(), None);
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, r#"{"aos_forfaitair":{}}"#);
        let back: BengAdjacency = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn aos_forfaitair_gevel_carries_orientation() {
        let a = BengAdjacency::AosForfaitair {
            orientatie: Some(Orientation::Zuid),
        };
        assert_eq!(a.orientatie(), Some(Orientation::Zuid));
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            r#"{"aos_forfaitair":{"orientatie":"zuid"}}"#
        );
    }

    #[test]
    fn adjacency_orientation_accessor() {
        assert_eq!(
            BengAdjacency::Buitenlucht {
                orientatie: Orientation::Zuid
            }
            .orientatie(),
            Some(Orientation::Zuid)
        );
        assert_eq!(BengAdjacency::Water.orientatie(), None);
        assert!(BengAdjacency::VloerOpMaaiveldBovenGrond.requires_omtrek());
        assert!(!BengAdjacency::VloerOpMaaiveldBovenKruipruimte.requires_omtrek());
    }

    #[test]
    fn adjacency_serializes_uniec_shaped() {
        // Oriëntatieloze variant → snake_case string.
        assert_eq!(
            serde_json::to_string(&BengAdjacency::VloerOpMaaiveldBovenGrond).unwrap(),
            "\"vloer_op_maaiveld_boven_grond\""
        );
        // Buitenlucht → extern getagd met oriëntatie.
        assert_eq!(
            serde_json::to_string(&BengAdjacency::Buitenlucht {
                orientatie: Orientation::Noord
            })
            .unwrap(),
            r#"{"buitenlucht":{"orientatie":"noord"}}"#
        );
    }

    #[test]
    fn rc_or_u_serializes_externally_tagged() {
        assert_eq!(serde_json::to_string(&RcOrU::Rc(3.70)).unwrap(), r#"{"rc":3.7}"#);
        assert_eq!(serde_json::to_string(&RcOrU::U(1.3)).unwrap(), r#"{"u":1.3}"#);
    }

    #[test]
    fn aantal_defaults_to_one() {
        let json = r#"{"kozijn_ref":"merk-a"}"#;
        let p: BengWindowPlacement = serde_json::from_str(json).unwrap();
        assert_eq!(p.aantal, 1);
        assert_eq!(p.belemmering, Obstruction::None);
        assert!(p.zonwering.is_none());
        assert!(!p.zomernachtventilatie);
    }

    #[test]
    fn placement_with_shading_round_trips() {
        let mut g = minimal_geo();
        g.zones[0].gevels[0].ramen[0].zonwering = Some(MovableSunShading {
            f_c: 0.35,
            control: ShadingControl::ManualResidential,
        });
        let json = serde_json::to_string(&g).unwrap();
        let back: BengGeometry = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}
