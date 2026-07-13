//! Fase 4a+4b — ZIP/BOM/serde-parsing en de entity/children-index.
//!
//! Het `.uniec3`-bestand is een PKZIP-archief met UTF-8-**met-BOM** JSON. Dit
//! module strip de BOM, deserialiseert de vier kernbestanden
//! (meta/buildings/entities/relations + summary) en bouwt de indexen waarop de
//! mappers lopen: een `data_id → entity`-lookup en een `parent → children`-graaf
//! uit `relations.json`.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::{Result, Uniec3ImportError};

// ---------------------------------------------------------------------------
// Rauwe archief-schema's
// ---------------------------------------------------------------------------

/// `meta.json` — containerversie + app-provenance.
#[derive(Debug, Deserialize)]
pub struct Meta {
    /// Containerformaat-versie. `2` voor alle bekende exports (2022–2025).
    #[serde(rename = "Version", default)]
    pub version: i64,
    /// App-identificatie, bv. `"NTA8800, Version=3.3.3.1, ..."`.
    #[serde(rename = "App", default)]
    pub app: Option<String>,
}

impl Meta {
    /// Extraheer de app-versie (`3.3.3.1`) uit het `App`-veld (`Version=x.y.z`).
    #[must_use]
    pub fn app_version(&self) -> Option<String> {
        let app = self.app.as_deref()?;
        let after = app.split("Version=").nth(1)?;
        let ver = after.split(',').next()?.trim();
        if ver.is_empty() {
            None
        } else {
            Some(ver.to_string())
        }
    }
}

/// Eén regel uit `buildings.json`.
#[derive(Debug, Deserialize)]
pub struct BuildingRef {
    /// Gebouw-id; bepaalt de `buildings/<id>/`-submap.
    #[serde(rename = "BuildingId")]
    pub building_id: i64,
}

/// Eén property (veldcode → waarde) op een entity.
#[derive(Debug, Clone, Deserialize)]
pub struct PropertyData {
    /// Veldcode, bv. `BEGR_A`.
    #[serde(rename = "NTAPropertyId")]
    pub id: String,
    /// Ruwe waarde (Nederlandse decimaalkomma-string, enum-code of GUID). Kan
    /// ontbreken; leeg/`n.v.t.`/`<none>` gelden als "niet gezet".
    #[serde(rename = "Value", default)]
    pub value: Option<String>,
}

/// Eén entity uit `entities.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Entity {
    /// Entity-type (de "tabelnaam"), bv. `BEGR`.
    #[serde(rename = "NTAEntityId")]
    pub entity_id: String,
    /// Instance-GUID (primary key).
    #[serde(rename = "NTAEntityDataId")]
    pub data_id: String,
    /// Volgorde binnen de parent.
    #[serde(rename = "Order", default)]
    pub order: f64,
    /// Property-lijst.
    #[serde(rename = "NTAPropertyDatas", default)]
    pub properties: Vec<PropertyData>,
}

impl Entity {
    /// Ruwe waarde van een veld, met leeg/`n.v.t.`/`<none>` → `None`.
    #[must_use]
    pub fn prop(&self, id: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|p| p.id == id)
            .and_then(|p| p.value.as_deref())
            .map(str::trim)
            .filter(|v| !is_empty_value(v))
    }

    /// Veldwaarde met `_NON`-terugval: de basis (user-override) wint; is die
    /// leeg, dan het forfaitaire `<id>_NON`. Zie formaat-analyse §3.
    #[must_use]
    pub fn prop_or_non(&self, id: &str) -> Option<&str> {
        if let Some(v) = self.prop(id) {
            return Some(v);
        }
        // `_NON`-sleutel apart opzoeken (geen format!-allocatie in de hot loop
        // hoeft: dit draait alleen op de installatie-mapper, niet per gevel).
        let non = format!("{id}_NON");
        self.properties
            .iter()
            .find(|p| p.id == non)
            .and_then(|p| p.value.as_deref())
            .map(str::trim)
            .filter(|v| !is_empty_value(v))
    }

    /// Parse een numeriek veld (Nederlandse komma) naar `f64`.
    #[must_use]
    pub fn num(&self, id: &str) -> Option<f64> {
        self.prop(id).and_then(parse_num)
    }

    /// Parse een numeriek veld met `_NON`-terugval.
    #[must_use]
    pub fn num_or_non(&self, id: &str) -> Option<f64> {
        self.prop_or_non(id).and_then(parse_num)
    }
}

/// Eén relatie-edge uit `relations.json`.
#[derive(Debug, Deserialize)]
pub struct Relation {
    /// Parent-instance-GUID.
    #[serde(rename = "ParentId")]
    pub parent_id: String,
    /// Child-instance-GUID.
    #[serde(rename = "ChildId")]
    pub child_id: String,
}

// ---------------------------------------------------------------------------
// Waarde-helpers
// ---------------------------------------------------------------------------

/// `true` als een ruwe waarde als "niet gezet" telt: leeg, `<none>` of
/// `n.v.t.` (Uniecs drie leeg-sentinels; zie formaat-analyse §3).
#[must_use]
pub fn is_empty_value(v: &str) -> bool {
    let t = v.trim();
    t.is_empty() || t.eq_ignore_ascii_case("<none>") || t.eq_ignore_ascii_case("n.v.t.")
}

/// Parse een Uniec-getal: Nederlandse decimaalkomma-string → `f64`.
///
/// Tolereert zowel `"21,96"` (komma) als `"0.98"` (punt — sommige `_DEFAULT`-
/// velden staan al met punt). Leeg/`n.v.t.`/`<none>` → `None`.
#[must_use]
pub fn parse_num(s: &str) -> Option<f64> {
    let t = s.trim();
    if is_empty_value(t) {
        return None;
    }
    t.replace(',', ".").parse::<f64>().ok()
}

// ---------------------------------------------------------------------------
// Archief lezen
// ---------------------------------------------------------------------------

/// Het uitgepakte archief: de gedeserialiseerde kernbestanden voor één gebouw.
pub struct RawArchive {
    /// `meta.json`.
    pub meta: Meta,
    /// Alle entities van het (eerste) gebouw.
    pub entities: Vec<Entity>,
    /// Alle relatie-edges van het gebouw.
    pub relations: Vec<Relation>,
    /// `summary.json` als losse waarde (certified-resultaten).
    pub summary: serde_json::Value,
}

/// Lees en deserialiseer een `.uniec3`-archief uit bytes.
///
/// # Errors
/// [`Uniec3ImportError::Zip`] bij een corrupt archief, [`MissingFile`] als een
/// kernbestand ontbreekt, [`Json`] bij ongeldige JSON, [`NoBuilding`] als
/// `buildings.json` leeg is.
///
/// [`MissingFile`]: Uniec3ImportError::MissingFile
/// [`Json`]: Uniec3ImportError::Json
/// [`NoBuilding`]: Uniec3ImportError::NoBuilding
pub fn read_archive(bytes: &[u8]) -> Result<RawArchive> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;

    // Zip-bomb-bescherming: begrens het aantal entries. Een echte `.uniec3` heeft
    // ~8 entries (4 top-level + 4 per gebouw); 1024 is ruim (>250 gebouwen) maar
    // sluit een gefabriceerd archief met miljoenen entries uit.
    if zip.len() > MAX_ENTRIES {
        return Err(Uniec3ImportError::TooManyEntries {
            count: zip.len(),
            limit: MAX_ENTRIES,
        });
    }

    let meta: Meta = read_json(&mut zip, "meta.json")?;
    let buildings: Vec<BuildingRef> = read_json(&mut zip, "buildings.json")?;
    let building = buildings.first().ok_or(Uniec3ImportError::NoBuilding)?;
    let bid = building.building_id;

    let entities: Vec<Entity> = read_json(&mut zip, &format!("buildings/{bid}/entities.json"))?;
    let relations: Vec<Relation> = read_json(&mut zip, &format!("buildings/{bid}/relations.json"))?;
    let summary: serde_json::Value = read_json(&mut zip, &format!("buildings/{bid}/summary.json"))?;

    Ok(RawArchive {
        meta,
        entities,
        relations,
        summary,
    })
}

/// Maximaal aantal archief-entries (zip-bomb-bescherming).
const MAX_ENTRIES: usize = 1024;

/// Maximale uitgepakte grootte per archief-entry: 64 MB. De grootste echte
/// `entities.json` in het korpus is ~580 KB, dus >100× marge; sluit een
/// gecomprimeerde zip-bomb uit.
const MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;

/// Lees één archief-entry, strip de BOM en deserialiseer naar `T`.
///
/// Beschermt tegen een oversized/zip-bomb-entry op twee niveaus: (1) een
/// pre-check op de gedeclareerde uitgepakte grootte (`ZipFile::size()`), en
/// (2) een harde bovengrens op het daadwerkelijk gelezen aantal bytes via
/// [`Read::take`] — zodat een liegende header ons nog steeds niet onbegrensd
/// laat decomprimeren.
fn read_json<R: Read + Seek, T: DeserializeOwned>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<T> {
    let file = zip
        .by_name(name)
        .map_err(|_| Uniec3ImportError::MissingFile(name.to_string()))?;

    // (1) Gedeclareerde uitgepakte grootte weren vóór het lezen.
    if file.size() > MAX_ENTRY_BYTES {
        return Err(Uniec3ImportError::EntryTooLarge {
            file: name.to_string(),
            size: file.size(),
            limit: MAX_ENTRY_BYTES,
        });
    }

    // (2) Lees begrensd; als de decompressie de limiet toch raakt (liegende
    // header), faal hard i.p.v. onbegrensd te bufferen.
    let mut buf = Vec::new();
    let read = file.take(MAX_ENTRY_BYTES + 1).read_to_end(&mut buf)? as u64;
    if read > MAX_ENTRY_BYTES {
        return Err(Uniec3ImportError::EntryTooLarge {
            file: name.to_string(),
            size: read,
            limit: MAX_ENTRY_BYTES,
        });
    }

    let slice = strip_bom(&buf);
    serde_json::from_slice(slice).map_err(|source| Uniec3ImportError::Json {
        file: name.to_string(),
        source,
    })
}

/// Strip een leidende UTF-8-BOM (`EF BB BF`), indien aanwezig.
fn strip_bom(buf: &[u8]) -> &[u8] {
    buf.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(buf)
}

// ---------------------------------------------------------------------------
// Entity/children-index
// ---------------------------------------------------------------------------

/// Geïndexeerde entity-graaf: `data_id → entity` + `parent → children`.
pub struct EntityIndex {
    entities: Vec<Entity>,
    by_id: HashMap<String, usize>,
    children: HashMap<String, Vec<usize>>,
}

impl EntityIndex {
    /// Bouw de index uit de rauwe entity- en relatie-lijsten.
    #[must_use]
    pub fn new(entities: Vec<Entity>, relations: Vec<Relation>) -> Self {
        let mut by_id = HashMap::with_capacity(entities.len());
        for (i, e) in entities.iter().enumerate() {
            by_id.insert(e.data_id.clone(), i);
        }
        let mut children: HashMap<String, Vec<usize>> = HashMap::new();
        for r in &relations {
            if let Some(&ci) = by_id.get(&r.child_id) {
                children.entry(r.parent_id.clone()).or_default().push(ci);
            }
        }
        Self {
            entities,
            by_id,
            children,
        }
    }

    /// Zoek een entity op instance-GUID.
    #[must_use]
    pub fn get(&self, data_id: &str) -> Option<&Entity> {
        self.by_id.get(data_id).map(|&i| &self.entities[i])
    }

    /// Alle entities van een gegeven type, op `Order` gesorteerd.
    #[must_use]
    pub fn of_type(&self, entity_type: &str) -> Vec<&Entity> {
        let mut v: Vec<&Entity> = self
            .entities
            .iter()
            .filter(|e| e.entity_id == entity_type)
            .collect();
        v.sort_by(|a, b| a.order.total_cmp(&b.order));
        v
    }

    /// De eerste entity van een type (laagste `Order`).
    #[must_use]
    pub fn first_of_type(&self, entity_type: &str) -> Option<&Entity> {
        self.of_type(entity_type).into_iter().next()
    }

    /// De directe children van `parent` met het gegeven type, op `Order`
    /// gesorteerd.
    #[must_use]
    pub fn children_of(&self, parent: &Entity, child_type: &str) -> Vec<&Entity> {
        let mut v: Vec<&Entity> = self
            .children
            .get(&parent.data_id)
            .into_iter()
            .flatten()
            .map(|&i| &self.entities[i])
            .filter(|e| e.entity_id == child_type)
            .collect();
        v.sort_by(|a, b| a.order.total_cmp(&b.order));
        v
    }

    /// De eerste child van `parent` met het gegeven type.
    #[must_use]
    pub fn child_of(&self, parent: &Entity, child_type: &str) -> Option<&Entity> {
        self.children_of(parent, child_type).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_num_handles_comma_and_dot() {
        assert_eq!(parse_num("21,96"), Some(21.96));
        assert_eq!(parse_num("0.98"), Some(0.98));
        assert_eq!(parse_num("1"), Some(1.0));
        assert_eq!(parse_num("90"), Some(90.0));
    }

    #[test]
    fn parse_num_rejects_empty_sentinels() {
        assert_eq!(parse_num(""), None);
        assert_eq!(parse_num("n.v.t."), None);
        assert_eq!(parse_num("<none>"), None);
    }

    #[test]
    fn empty_value_sentinels() {
        assert!(is_empty_value(""));
        assert!(is_empty_value("N.V.T."));
        assert!(is_empty_value("<none>"));
        assert!(!is_empty_value("GVL_BTNL_N"));
    }

    #[test]
    fn app_version_parsed_from_meta() {
        let m = Meta {
            version: 2,
            app: Some("NTA8800, Version=3.3.3.1, Culture=neutral".to_string()),
        };
        assert_eq!(m.app_version().as_deref(), Some("3.3.3.1"));
    }

    fn entity(id: &str, data: &str, props: &[(&str, &str)]) -> Entity {
        Entity {
            entity_id: id.to_string(),
            data_id: data.to_string(),
            order: 100.0,
            properties: props
                .iter()
                .map(|(k, v)| PropertyData {
                    id: (*k).to_string(),
                    value: Some((*v).to_string()),
                })
                .collect(),
        }
    }

    #[test]
    fn prop_or_non_falls_back() {
        let e = entity("VERW-OPWEK", "a", &[("X", ""), ("X_NON", "4,10")]);
        assert_eq!(e.prop_or_non("X"), Some("4,10"));
        assert_eq!(e.num_or_non("X"), Some(4.10));
    }

    #[test]
    fn index_links_children() {
        let parent = entity("BEGR", "p1", &[]);
        let child = entity("CONSTRD", "c1", &[("CONSTRD_OPP", "10,0")]);
        let idx = EntityIndex::new(
            vec![parent, child],
            vec![Relation {
                parent_id: "p1".to_string(),
                child_id: "c1".to_string(),
            }],
        );
        let p = idx.get("p1").unwrap();
        let kids = idx.children_of(p, "CONSTRD");
        assert_eq!(kids.len(), 1);
        assert_eq!(kids[0].num("CONSTRD_OPP"), Some(10.0));
    }
}
