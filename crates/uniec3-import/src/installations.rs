//! Fase 4d — entity-graaf → [`EnergyInput`] + `q_v10;spec`.
//!
//! Loopt de installatie-subtrees onder `INSTALLATIE` (VERW/TAPW/VENT/KOEL/PV) en
//! de infiltratie onder `UNIT → INFILUNIT`. Zie formaat-analyse §5b. De
//! enum-mapping is best-effort met tolerante terugval + waarschuwing; de
//! kern-validatie van F8 zit op de geometrie + certified, niet hier.
//!
//! **PV-piekvermogen (open vraag 1 uit de analyse):** we hanteren het
//! **veld-totaal** `aantal_panelen × Wp/paneel` (PM-besluit: verwachte
//! certificerings-grondslag per PV-VELD). Het productblad-totaal `PV_WPPRDT`
//! wijkt daarvan af (bv. 6736 vs 10×410=4100 Wp) en wordt als waarschuwing
//! meegegeven zodat de afwijkende definitie herleidbaar blijft.

use openaec_project_shared::energy::{
    CoolingGeneratorType, CoolingInput, DhwGeneratorType, DhwInput, EnergyInput, HeatEmissionType,
    HeatGeneratorType, HeatingInput, PvInput, VentilationInput, VentilationSystemType,
};

use crate::parse::{Entity, EntityIndex};

/// Map de installaties + infiltratie. Retourneert het [`EnergyInput`]-blok en de
/// specifieke luchtdoorlatendheid `q_v10;spec` (dm³/(s·m²) per A_g) voor
/// `SharedProject`.
pub fn map_installations(
    idx: &EntityIndex,
    warnings: &mut Vec<String>,
) -> (EnergyInput, Option<f64>) {
    let mut energy = EnergyInput::default();

    for inst in idx.of_type("INSTALLATIE") {
        match inst.prop("INSTALL_TYPE") {
            Some("INST_VERW") => energy.heating = map_heating(idx, inst, warnings),
            Some("INST_TAPW") => energy.dhw = map_dhw(idx, inst, warnings),
            Some("INST_VENT") => energy.ventilation = map_ventilation(idx, inst, warnings),
            Some("INST_KOEL") => energy.cooling = map_cooling(idx, inst, warnings),
            Some("INST_PV") => energy.pv = map_pv(idx, inst, warnings),
            other => warnings.push(format!("onbekend INSTALL_TYPE {other:?} → overgeslagen")),
        }
    }

    let q_v10 = map_q_v10(idx);

    (energy, q_v10)
}

/// `q_v10;spec` uit de eerste gevulde `INFILUNIT` (INFILUNIT_QV).
fn map_q_v10(idx: &EntityIndex) -> Option<f64> {
    idx.of_type("INFILUNIT")
        .into_iter()
        .find_map(|e| e.num("INFILUNIT_QV"))
}

// ---------------------------------------------------------------------------
// Verwarming
// ---------------------------------------------------------------------------

fn map_heating(idx: &EntityIndex, inst: &Entity, warnings: &mut Vec<String>) -> Option<HeatingInput> {
    let verw = idx.child_of(inst, "VERW")?;
    let opwek = idx.child_of(verw, "VERW-OPWEK");
    let afg = idx.child_of(verw, "VERW-AFG");

    let generator = match opwek.and_then(|o| o.prop("VERW-OPWEK_POMP")) {
        Some("VERW-OPWEK_POMP_BUWA") => HeatGeneratorType::HeatPumpAir,
        Some(code) if code.contains("BOWA") || code.contains("BODEM") => {
            HeatGeneratorType::HeatPumpGround
        }
        Some(other) => {
            warnings.push(format!(
                "verwarming: onbekende warmtepomp-bron {other} → lucht/water aangenomen"
            ));
            HeatGeneratorType::HeatPumpAir
        }
        None => {
            // Geen warmtepomp-bron → ketel (HR) als tolerante terugval.
            HeatGeneratorType::HrBoiler
        }
    };

    let emission = afg.and_then(|a| a.prop("VERW-AFG_TYPE_AFG")).map(|code| match code {
        "VERW-AFG_TYPE_AFG_VLV" => HeatEmissionType::FloorHeating,
        "VERW-AFG_TYPE_AFG_RADLT" => HeatEmissionType::RadiatorLowTemp,
        "VERW-AFG_TYPE_AFG_RADHT" => HeatEmissionType::RadiatorHighTemp,
        "VERW-AFG_TYPE_AFG_LUCHT" => HeatEmissionType::AirHeating,
        other => {
            warnings.push(format!("verwarming: onbekend afgiftetype {other} → radiator HT"));
            HeatEmissionType::RadiatorHighTemp
        }
    });

    Some(HeatingInput {
        generator,
        cop: opwek.and_then(|o| o.num_or_non("VERW-OPWEK_COP")),
        hr_class: None,
        district_factor: None,
        emission,
        distribution_efficiency: None,
        control_factor: None,
        coverage_fraction: 1.0,
        source: None,
    })
}

// ---------------------------------------------------------------------------
// Tapwater
// ---------------------------------------------------------------------------

fn map_dhw(idx: &EntityIndex, inst: &Entity, warnings: &mut Vec<String>) -> Option<DhwInput> {
    let tapw = idx.child_of(inst, "TAPW")?;
    let opwek = idx.child_of(tapw, "TAPW-OPWEK");

    let generator = match opwek.and_then(|o| o.prop("TAPW-OPWEK_BRON_POMP")) {
        Some(_) => DhwGeneratorType::HeatPump,
        None => match opwek.and_then(|o| o.prop("TAPW-OPWEK_TYPE")) {
            Some(code) if code.contains("ELEK") => DhwGeneratorType::ElectricBoiler,
            Some(_) | None => DhwGeneratorType::HrCombiBoiler,
        },
    };

    // Rendement: SCOP_W (warmtepomp, COP_NON) of η_gen (ketel, REND_NON).
    let efficiency = opwek.and_then(|o| {
        o.num_or_non("TAPW-OPWEK_COP")
            .or_else(|| o.num("TAPW-OPWEK_REND_NON"))
    });

    let _ = warnings; // dhw-mapping heeft (nog) geen eigen waarschuwingen
    Some(DhwInput {
        generator,
        efficiency,
        dwtw: None,
        has_solar_boiler: false,
        solar_boiler_fraction: None,
        source: None,
    })
}

// ---------------------------------------------------------------------------
// Ventilatie
// ---------------------------------------------------------------------------

fn map_ventilation(
    idx: &EntityIndex,
    inst: &Entity,
    warnings: &mut Vec<String>,
) -> Option<VentilationInput> {
    let vent = idx.child_of(inst, "VENT")?;

    // Systeemtype uit VENT_VARIANT (VARIANT_D2 → D); terugval op VENT_SYS.
    let system = vent
        .prop("VENT_VARIANT")
        .and_then(|v| v.strip_prefix("VARIANT_"))
        .and_then(|s| s.chars().next())
        .and_then(letter_to_system)
        .or_else(|| match vent.prop("VENT_SYS") {
            Some("VENTSYS_NAT") => Some(VentilationSystemType::A),
            Some("VENTSYS_MECHT") => Some(VentilationSystemType::B),
            Some("VENTSYS_MECHC") => Some(VentilationSystemType::C),
            Some("VENTSYS_BALANS") => Some(VentilationSystemType::D),
            _ => None,
        })
        .unwrap_or_else(|| {
            warnings.push(format!(
                "ventilatie: onbekend systeem (VENT_VARIANT={:?}, VENT_SYS={:?}) → D aangenomen",
                vent.prop("VENT_VARIANT"),
                vent.prop("VENT_SYS")
            ));
            VentilationSystemType::D
        });

    // WTW-rendement uit de eerste WARMTETERUG met een actieve WTW.
    let wtw_efficiency = idx.children_of(vent, "WARMTETERUG").into_iter().find_map(|w| {
        match w.prop("WARMTETERUG_WTW") {
            Some("WARMTETERUG_WTW_NIET") | None => None,
            Some(_) => w.num("WARMTETERUG_REND"),
        }
    });

    Some(VentilationInput {
        system,
        wtw_efficiency,
        sfp_w_per_m3h: None,
        bypass_enabled: false,
        mechanical_supply_m3_per_h: None,
        mechanical_exhaust_m3_per_h: None,
        infiltration_m3_per_h: None,
        q_v10_spec_dm3_s_m2: None,
        source: None,
    })
}

fn letter_to_system(c: char) -> Option<VentilationSystemType> {
    match c {
        'A' => Some(VentilationSystemType::A),
        'B' => Some(VentilationSystemType::B),
        'C' => Some(VentilationSystemType::C),
        'D' => Some(VentilationSystemType::D),
        'E' => Some(VentilationSystemType::E),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Koeling
// ---------------------------------------------------------------------------

fn map_cooling(idx: &EntityIndex, inst: &Entity, _warnings: &mut [String]) -> Option<CoolingInput> {
    let koel = idx.child_of(inst, "KOEL")?;
    let opwek = idx.child_of(koel, "KOEL-OPWEK");

    Some(CoolingInput {
        generator: CoolingGeneratorType::Compression,
        seer: opwek.and_then(|o| o.num_or_non("KOEL-OPWEK_EER")),
        cop: None,
        free_cooling_fraction: None,
        source: None,
    })
}

// ---------------------------------------------------------------------------
// PV
// ---------------------------------------------------------------------------

fn map_pv(idx: &EntityIndex, inst: &Entity, warnings: &mut Vec<String>) -> Vec<PvInput> {
    let Some(pv) = idx.child_of(inst, "PV") else {
        return Vec::new();
    };
    let wp_per_panel = pv.num_or_non("PV_WPPNL");

    if let Some(prdt) = pv.num("PV_WPPRDT") {
        warnings.push(format!(
            "PV: piekvermogen op veld-totaal (aantal×Wp/paneel); productblad-totaal PV_WPPRDT={prdt:.0} Wp wijkt af en is niet gebruikt"
        ));
    }

    let mut fields = Vec::new();
    for (i, veld) in idx.children_of(pv, "PV-VELD").into_iter().enumerate() {
        // Ontbrekend aantal panelen → skip+warn (stil 0 zou BENG 3 vertekenen).
        let Some(aantal) = veld.num("PV-VELD_AANTALPNL") else {
            warnings.push(format!(
                "PV-VELD {i}: geen aantal panelen (PV-VELD_AANTALPNL) → veld overgeslagen"
            ));
            continue;
        };
        let Some(wp) = wp_per_panel else {
            warnings.push("PV: geen Wp/paneel (PV_WPPNL) → veld overgeslagen".to_string());
            continue;
        };
        let peak_kwp = aantal * wp / 1000.0;
        let azimuth = pv_orientation_deg(veld.prop("PV-VELD_ORIE")).unwrap_or_else(|| {
            warnings.push(format!(
                "PV-VELD {}: onbekende oriëntatie {:?} → zuid (180°)",
                i,
                veld.prop("PV-VELD_ORIE")
            ));
            180.0
        });
        // Ontbrekende helling → default 0° (horizontaal) mét waarschuwing.
        let tilt = veld.num("PV-VELD_HELLING").unwrap_or_else(|| {
            warnings.push(format!(
                "PV-VELD {i}: geen helling (PV-VELD_HELLING) → 0° (horizontaal) aangenomen"
            ));
            0.0
        });
        fields.push(PvInput {
            id: Some(veld.data_id.clone()),
            name: None,
            peak_power_kwp: peak_kwp,
            azimuth_degrees: azimuth,
            tilt_degrees: tilt,
            system_efficiency: None,
            inverter_efficiency: None,
            shadow_factor: None,
            source: None,
        });
    }
    fields
}

/// `PVORIE_N`/`_O`/`_Z`/`_W`(+ diagonalen) → azimut in graden (0 = noord).
fn pv_orientation_deg(code: Option<&str>) -> Option<f64> {
    let suffix = code?.strip_prefix("PVORIE_")?;
    Some(match suffix {
        "N" => 0.0,
        "NO" => 45.0,
        "O" => 90.0,
        "ZO" => 135.0,
        "Z" => 180.0,
        "ZW" => 225.0,
        "W" => 270.0,
        "NW" => 315.0,
        "HOR" => 180.0, // horizontaal veld: azimut niet relevant, zuid als neutraal
        _ => return None,
    })
}
