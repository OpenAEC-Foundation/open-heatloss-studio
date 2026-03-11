//! IFCX (IFC5) document types.
//!
//! Mirrors the IFC5 alpha spec from buildingSMART/IFC5-development.
//! An IFCX document is a flat list of data entries identified by path,
//! with typed attributes grouped by namespace.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level IFCX document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfcxDocument {
    pub header: IfcxHeader,
    pub imports: Vec<IfcxImport>,
    #[serde(default)]
    pub schemas: HashMap<String, serde_json::Value>,
    pub data: Vec<IfcxDataEntry>,
}

/// Document header with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcxHeader {
    pub id: String,
    pub ifcx_version: String,
    pub data_version: String,
    pub author: String,
    pub timestamp: String,
}

/// Schema import reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfcxImport {
    pub uri: String,
}

/// A single data entry in the IFCX flat list.
///
/// Each entry is identified by a unique `path` (typically a UUID).
/// `children` maps named slots to other entry paths.
/// `attributes` holds namespaced data (e.g. `bsi::ifc::class`, `isso51::calc::result`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfcxDataEntry {
    pub path: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub children: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inherits: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Standard IFC namespace constants
// ---------------------------------------------------------------------------

/// Standard buildingSMART IFC namespaces.
pub mod ns {
    pub const IFC_CLASS: &str = "bsi::ifc::class";
    pub const IFC_PROP: &str = "bsi::ifc::prop";
}

/// IFC class codes used in `bsi::ifc::class` attributes.
pub mod ifc_class {
    pub const PROJECT: &str = "IfcProject";
    pub const SITE: &str = "IfcSite";
    pub const BUILDING: &str = "IfcBuilding";
    pub const BUILDING_STOREY: &str = "IfcBuildingStorey";
    pub const SPACE: &str = "IfcSpace";
    pub const WALL: &str = "IfcWall";
    pub const WINDOW: &str = "IfcWindow";
    pub const DOOR: &str = "IfcDoor";
    pub const SLAB: &str = "IfcSlab";
    pub const ROOF: &str = "IfcRoof";
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl IfcxDocument {
    /// Create a new empty IFCX document.
    pub fn new(author: &str) -> Self {
        Self {
            header: IfcxHeader {
                id: uuid::Uuid::new_v4().to_string(),
                ifcx_version: "ifcx_alpha".to_string(),
                data_version: "1.0.0".to_string(),
                author: author.to_string(),
                timestamp: String::new(), // caller should set
            },
            imports: vec![
                IfcxImport {
                    uri: "https://ifcx.dev/@standards.buildingsmart.org/ifc/core/ifc@v5a.ifcx"
                        .to_string(),
                },
                IfcxImport {
                    uri: "https://ifcx.dev/@standards.buildingsmart.org/ifc/core/prop@v5a.ifcx"
                        .to_string(),
                },
            ],
            schemas: HashMap::new(),
            data: Vec::new(),
        }
    }

    /// Find a data entry by path.
    pub fn find(&self, path: &str) -> Option<&IfcxDataEntry> {
        self.data.iter().find(|e| e.path == path)
    }

    /// Find all entries that have a specific IFC class.
    pub fn find_by_class(&self, class_code: &str) -> Vec<&IfcxDataEntry> {
        self.data
            .iter()
            .filter(|e| {
                e.attributes
                    .get(ns::IFC_CLASS)
                    .and_then(|v| v.get("code"))
                    .and_then(|v| v.as_str())
                    == Some(class_code)
            })
            .collect()
    }
}

impl IfcxDataEntry {
    /// Create a new entry with the given path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            children: HashMap::new(),
            inherits: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    /// Get a typed attribute value.
    pub fn get_attr<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.attributes
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a typed attribute value.
    pub fn set_attr<T: Serialize>(&mut self, key: &str, value: &T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.attributes.insert(key.to_string(), v);
        }
    }

    /// Get the IFC class code (e.g. "IfcSpace") if present.
    pub fn ifc_class(&self) -> Option<&str> {
        self.attributes
            .get(ns::IFC_CLASS)
            .and_then(|v| v.get("code"))
            .and_then(|v| v.as_str())
    }

    /// Get an IFC property value.
    pub fn ifc_prop(&self, name: &str) -> Option<&serde_json::Value> {
        let key = format!("{}::{}", ns::IFC_PROP, name);
        self.attributes.get(&key)
    }
}

// ---------------------------------------------------------------------------
// Composition: merge multiple IFCX documents
// ---------------------------------------------------------------------------

/// Compose multiple IFCX documents into a single flat entry list.
///
/// Later entries override earlier ones (layer composition).
/// This is the Rust equivalent of the TypeScript `composeIfcxDocuments()`.
pub fn compose(docs: &[&IfcxDocument]) -> Vec<IfcxDataEntry> {
    let mut merged: HashMap<String, IfcxDataEntry> = HashMap::new();

    for doc in docs {
        for entry in &doc.data {
            if let Some(existing) = merged.get_mut(&entry.path) {
                existing.children.extend(entry.children.clone());
                existing.inherits.extend(entry.inherits.clone());
                existing.attributes.extend(entry.attributes.clone());
            } else {
                merged.insert(entry.path.clone(), entry.clone());
            }
        }
    }

    merged.into_values().collect()
}
