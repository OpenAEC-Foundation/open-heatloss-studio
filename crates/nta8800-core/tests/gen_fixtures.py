"""Genereer de 20 voorbeeld-fixtures voor nta8800-core.

Eenmalig draaien vanuit crates/nta8800-core/:

    python tests/gen_fixtures.py

Schrijft tests/fixtures/*.json — 10 woningbouw + 10 utiliteit varianten die
samen alle installatie-types (HR-ketel, warmtepomp, elektrisch, stadswarmte),
ventilatie-systemen (A/B/C/D+WTW), koeling (compressie/absorptie/vrij), PV en
douche-WTW dekken. De integratietest `examples_test.rs` draait ze allemaal.
"""
import json
from pathlib import Path

OUT = Path(__file__).parent / "fixtures"
OUT.mkdir(exist_ok=True)


def win(id_, area, u=1.4, g=0.6, ff=0.25):
    return {
        "id": id_,
        "area_m2": area,
        "u_value": u,
        "g_value": g,
        "frame_fraction": ff,
    }


def wall(id_, area, u, orient=None, windows=None, boundary=None, tilt=90.0):
    return {
        "id": id_,
        "description": id_,
        "area_m2": area,
        "u_value": u,
        "boundary": boundary or {"type": "exterior"},
        "orientation_deg": orient,
        "tilt_deg": tilt,
        "windows": windows or [],
    }


def envelope_woning(scale=1.0, u_wall=0.30, u_roof=0.20, u_floor=0.25, u_win=1.4,
                    win_zuid=8.0, win_noord=4.0):
    """Standaard woning-envelope: 4 gevels + dak + grondvloer, geschaald."""
    return [
        wall("gevel-zuid", 40 * scale, u_wall, 180.0,
             [win("raam-zuid", win_zuid * scale, u_win)]),
        wall("gevel-noord", 40 * scale, u_wall, 0.0,
             [win("raam-noord", win_noord * scale, u_win)]),
        wall("gevel-oost", 25 * scale, u_wall, 90.0,
             [win("raam-oost", 2.0 * scale, u_win)]),
        wall("gevel-west", 25 * scale, u_wall, 270.0,
             [win("raam-west", 2.0 * scale, u_win)]),
        wall("dak", 70 * scale, u_roof, None, [], None, 45.0),
        wall("bg-vloer", 60 * scale, u_floor, None, [],
             {"type": "ground"}, 0.0),
    ]


def envelope_utiliteit(floor_area, storeys=2, u_wall=0.35, u_roof=0.25,
                       u_floor=0.30, u_win=1.6, glass_frac=0.30):
    """Utiliteit-envelope: doos-vormig, glas-fractie per gevel."""
    import math
    footprint = floor_area / storeys
    side = math.sqrt(footprint)
    height = storeys * 3.5
    facade = side * height
    win_area = facade * glass_frac
    return [
        wall("gevel-zuid", facade, u_wall, 180.0,
             [win("glas-zuid", win_area, u_win)]),
        wall("gevel-noord", facade, u_wall, 0.0,
             [win("glas-noord", win_area * 0.7, u_win)]),
        wall("gevel-oost", facade, u_wall, 90.0,
             [win("glas-oost", win_area * 0.5, u_win)]),
        wall("gevel-west", facade, u_wall, 270.0,
             [win("glas-west", win_area * 0.5, u_win)]),
        wall("dak", footprint, u_roof, None, [], None, 0.0),
        wall("bg-vloer", footprint, u_floor, None, [],
             {"type": "ground"}, 0.0),
    ]


def gas_hr(klasse="hr107", emission="radiator_low_temp"):
    # serde snake_case van HRClass::HR107 is "h_r107"
    klasse = {"hr100": "h_r100", "hr104": "h_r104", "hr107": "h_r107"}.get(klasse, klasse)
    return {
        "emission": {"type": emission},
        "generation": {"type": "h_r_boiler", "class": klasse},
        "distribution_efficiency": 0.95,
        "control_factor": 0.97,
    }


def heat_pump(scop, emission="floor_heating"):
    return {
        "emission": {"type": emission},
        "generation": {"type": "heat_pump", "scop": scop},
        "distribution_efficiency": 0.97,
        "control_factor": 0.98,
    }


def dhw_gas():
    return {
        "generation": {"type": "h_r_combi_boiler"},
        "emission": {"type": "woning_default"},
        "distribution_efficiency": 1.0,
        "shower_heat_recovery": None,
    }


def dhw_electric():
    return {
        "generation": {"type": "electric_boiler", "storage_loss_factor": 0.90},
        "emission": {"type": "woning_default"},
        "distribution_efficiency": 1.0,
        "shower_heat_recovery": None,
    }


def dhw_hp(scop=2.5, douche_wtw=None):
    d = {
        "generation": {"type": "heat_pump_dhw", "scop_dhw": scop},
        "emission": {"type": "woning_default"},
        "distribution_efficiency": 1.0,
        "shower_heat_recovery": None,
    }
    if douche_wtw:
        d["shower_heat_recovery"] = {
            "efficiency": douche_wtw,
            "douche_aandeel": 0.4,
        }
    return d


def dhw_district(factor=0.95, emission="woning_default"):
    return {
        "generation": {"type": "district_heating", "factor": factor},
        "emission": {"type": emission},
        "distribution_efficiency": 0.85,
        "shower_heat_recovery": None,
    }


def pv(kwp, azimuth=180.0, tilt=35.0):
    return [{
        "peak_power_kwp": kwp,
        "tilt_degrees": tilt,
        "azimuth_degrees": azimuth,
        "system_efficiency": 0.85,
        "inverter_efficiency": 0.96,
        "shadow_factor": 1.0,
    }]


def led_lighting(p=8.0, fu=0.285, fd=0.85, fc=0.9):
    return {
        "installed_power_w_per_m2": p,
        "utilization_factor": fu,
        "daylight_factor": fd,
        "control_factor": fc,
    }


def project(name, usage, area, envelope, heating, dhw, volume=None, year=2000,
             mass="heavy", vent=None, cooling=None, lighting=None, pv_sys=None):
    return {
        "info": {"name": name, "description": None},
        "building": {
            "usage_function": usage,
            "floor_area_m2": area,
            "volume_m3": volume or area * 2.7,
            "construction_year": year,
            "thermal_mass": mass,
        },
        "envelope": envelope,
        "ventilation": vent or {"system": {"type": "mechanical_exhaust"}},
        "heating": heating,
        "cooling": cooling,
        "dhw": dhw,
        "lighting": lighting,
        "pv": pv_sys,
        "conditions": {
            "heating_setpoint_c": 20.0,
            "cooling_setpoint_c": 24.0,
            "shading_factor": 1.0,
        },
    }


FIXTURES = {
    # ---------------- Woningbouw (10) ----------------
    "w01-tussenwoning-gas-hr107": project(
        "w01 Tussenwoning gas HR-107", "woonfunctie", 110.0,
        envelope_woning(1.0, u_wall=0.40, u_win=1.6),
        gas_hr("hr107"), dhw_gas(), year=1992),

    "w02-vrijstaand-oud-gas-hr100": project(
        "w02 Vrijstaand 1975 gas HR-100", "woonfunctie", 150.0,
        envelope_woning(1.35, u_wall=1.4, u_roof=0.9, u_floor=1.7, u_win=2.9),
        gas_hr("hr100", "radiator_high_temp"), dhw_gas(), year=1975,
        vent={"system": {"type": "natural"}}),

    "w03-appartement-elektrisch": project(
        "w03 Appartement volledig elektrisch", "woonfunctie", 55.0,
        [
            wall("gevel-zuid", 18, 0.35, 180.0, [win("raam-z", 5.0, 1.4)]),
            wall("gevel-noord", 18, 0.35, 0.0, [win("raam-n", 3.0, 1.4)]),
            wall("wand-trappenhuis", 20, 0.8, None, [],
                 {"type": "unheated_space", "id": "trappenhuis"}, 90.0),
        ],
        {
            "emission": {"type": "radiator_low_temp"},
            "generation": {"type": "electric_resistance"},
            "distribution_efficiency": 1.0,
            "control_factor": 0.97,
        },
        dhw_electric(), year=1985),

    "w04-hoekwoning-wp-lucht-wtw": project(
        "w04 Hoekwoning lucht-WP + WTW", "woonfunctie", 120.0,
        envelope_woning(1.05, u_wall=0.25, u_roof=0.16, u_win=1.2),
        heat_pump(4.0), dhw_hp(2.5), year=2015,
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.85}}),

    "w05-nieuwbouw-wp-bodem-pv": project(
        "w05 Nieuwbouw bodem-WP + 6 kWp PV + douche-WTW", "woonfunctie", 140.0,
        envelope_woning(1.2, u_wall=0.15, u_roof=0.12, u_floor=0.15, u_win=0.9),
        heat_pump(5.0), dhw_hp(3.0, douche_wtw=0.35), year=2023, mass="light",
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.90}},
        pv_sys=pv(6.0)),

    "w06-portiek-stadswarmte": project(
        "w06 Portiekwoning stadswarmte", "woonfunctie", 75.0,
        [
            wall("gevel-zuid", 22, 0.45, 180.0, [win("raam-z", 6.0, 1.8)]),
            wall("gevel-noord", 22, 0.45, 0.0, [win("raam-n", 4.0, 1.8)]),
            wall("wand-buren", 35, 0.6, None, [],
                 {"type": "unheated_space", "id": "buren"}, 90.0),
        ],
        {
            "emission": {"type": "radiator_high_temp"},
            "generation": {"type": "district_heating", "factor": 0.95},
            "distribution_efficiency": 0.90,
            "control_factor": 0.95,
        },
        dhw_district(), year=1998),

    "w07-bungalow-koeling": project(
        "w07 Bungalow met compressie-koeling", "woonfunctie", 130.0,
        envelope_woning(1.15, u_wall=0.30, win_zuid=20.0),
        gas_hr("hr107"), dhw_gas(), year=2005,
        cooling={
            "system": {"type": "compression_cooling", "scop_cooling": 3.5},
            "distribution": {"efficiency": 0.95},
            "emission": {"efficiency": 0.95, "regulation_factor": 0.95},
        }),

    "w08-rijwoning-wp-vrije-koeling": project(
        "w08 Rijwoning WP + vrije koeling", "woonfunctie", 105.0,
        envelope_woning(0.95, u_wall=0.28, u_win=1.3),
        heat_pump(3.5, "radiator_low_temp"), dhw_hp(2.2), year=2010,
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.80}},
        cooling={
            "system": {"type": "free_cooling", "factor": 0.3},
            "distribution": {"efficiency": 0.98},
            "emission": {"efficiency": 0.97, "regulation_factor": 0.97},
        }),

    "w09-herenhuis-gas-pv": project(
        "w09 Herenhuis gas HR-104 + 3 kWp PV", "woonfunctie", 220.0,
        envelope_woning(1.7, u_wall=0.60, u_roof=0.40, u_win=1.8),
        gas_hr("hr104", "radiator_high_temp"), dhw_gas(), year=1930,
        pv_sys=pv(3.0, azimuth=135.0)),

    "w10-tiny-house": project(
        "w10 Tiny house elektrisch", "woonfunctie", 30.0,
        [
            wall("gevel-zuid", 12, 0.20, 180.0, [win("raam-z", 4.0, 1.0)]),
            wall("gevel-noord", 12, 0.20, 0.0, []),
            wall("gevel-oost", 8, 0.20, 90.0, [win("raam-o", 1.0, 1.0)]),
            wall("gevel-west", 8, 0.20, 270.0, []),
            wall("dak", 32, 0.15, None, [], None, 10.0),
            wall("vloer", 30, 0.20, None, [], {"type": "ground"}, 0.0),
        ],
        {
            "emission": {"type": "air_heating"},
            "generation": {"type": "electric_resistance"},
            "distribution_efficiency": 1.0,
            "control_factor": 0.97,
        },
        dhw_electric(), year=2022, mass="light",
        vent={"system": {"type": "natural"}}),

    # ---------------- Utiliteit (10) ----------------
    "u01-kantoor-klein": project(
        "u01 Kantoor 400 m² gas + LED", "kantoorfunctie", 400.0,
        envelope_utiliteit(400.0, storeys=2),
        gas_hr("hr107", "radiator_low_temp"),
        dhw_district(0.95, "utiliteit_kort") | {"generation": {"type": "h_r_combi_boiler"}},
        volume=400 * 3.5, year=2008,
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.75}},
        lighting=led_lighting()),

    "u02-kantoor-groot-pv-koeling": project(
        "u02 Kantoor 2000 m² WP + 50 kWp PV + koeling", "kantoorfunctie", 2000.0,
        envelope_utiliteit(2000.0, storeys=4, u_wall=0.25, u_win=1.2, glass_frac=0.4),
        heat_pump(4.5, "air_heating"),
        {
            "generation": {"type": "heat_pump_dhw", "scop_dhw": 2.8},
            "emission": {"type": "utiliteit_kort"},
            "distribution_efficiency": 0.90,
            "shower_heat_recovery": None,
        },
        volume=2000 * 3.5, year=2020, mass="light",
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.85}},
        cooling={
            "system": {"type": "compression_cooling", "scop_cooling": 4.0},
            "distribution": {"efficiency": 0.95},
            "emission": {"efficiency": 0.95, "regulation_factor": 0.95},
        },
        lighting=led_lighting(6.0, 0.285, 0.6, 0.9),
        pv_sys=pv(50.0, tilt=15.0)),

    "u03-school": project(
        "u03 School 1200 m² gas", "onderwijsfunctie", 1200.0,
        envelope_utiliteit(1200.0, storeys=2, glass_frac=0.35),
        gas_hr("hr107", "radiator_high_temp"),
        {
            "generation": {"type": "h_r_combi_boiler"},
            "emission": {"type": "utiliteit_lang"},
            "distribution_efficiency": 0.85,
            "shower_heat_recovery": None,
        },
        volume=1200 * 3.8, year=1995,
        vent={"system": {"type": "mechanical_exhaust"}},
        lighting=led_lighting(9.0, 0.30, 0.8, 0.95)),

    "u04-winkel": project(
        "u04 Winkel 300 m² elektrisch + koeling", "winkelfunctie", 300.0,
        envelope_utiliteit(300.0, storeys=1, glass_frac=0.5),
        {
            "emission": {"type": "air_heating"},
            "generation": {"type": "electric_resistance"},
            "distribution_efficiency": 1.0,
            "control_factor": 0.95,
        },
        {
            "generation": {"type": "electric_boiler", "storage_loss_factor": 0.9},
            "emission": {"type": "utiliteit_kort"},
            "distribution_efficiency": 1.0,
            "shower_heat_recovery": None,
        },
        volume=300 * 4.0, year=2012,
        cooling={
            "system": {"type": "compression_cooling", "scop_cooling": 3.0},
            "distribution": {"efficiency": 0.95},
            "emission": {"efficiency": 0.92, "regulation_factor": 0.95},
        },
        lighting=led_lighting(14.0, 0.45, 1.0, 0.95)),

    "u05-zorgcentrum": project(
        "u05 Zorgcentrum 800 m² stadswarmte + WTW", "gezondheidszorgfunctie", 800.0,
        envelope_utiliteit(800.0, storeys=3, u_wall=0.30),
        {
            "emission": {"type": "radiator_low_temp"},
            "generation": {"type": "district_heating", "factor": 0.95},
            "distribution_efficiency": 0.88,
            "control_factor": 0.95,
        },
        dhw_district(0.95, "utiliteit_lang"),
        volume=800 * 3.2, year=2016,
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.80}},
        lighting=led_lighting(10.0, 0.55, 0.9, 0.95)),

    "u06-hotel": project(
        "u06 Hotel 600 m² gas + douche-WTW", "logiesfunctie", 600.0,
        envelope_utiliteit(600.0, storeys=3, glass_frac=0.25),
        gas_hr("hr107", "radiator_high_temp"),
        {
            "generation": {"type": "h_r_combi_boiler"},
            "emission": {"type": "utiliteit_lang"},
            "distribution_efficiency": 0.80,
            "shower_heat_recovery": {"efficiency": 0.30, "douche_aandeel": 0.6},
        },
        volume=600 * 3.0, year=2003,
        lighting=led_lighting(7.0, 0.50, 0.9, 0.9)),

    "u07-sporthal": project(
        "u07 Sporthal 900 m² gas HR-100", "sportfunctie", 900.0,
        envelope_utiliteit(900.0, storeys=1, u_wall=0.40, u_roof=0.30,
                           glass_frac=0.10),
        gas_hr("hr100", "air_heating"),
        {
            "generation": {"type": "h_r_combi_boiler"},
            "emission": {"type": "utiliteit_lang"},
            "distribution_efficiency": 0.85,
            "shower_heat_recovery": {"efficiency": 0.30, "douche_aandeel": 0.8},
        },
        volume=900 * 7.0, year=1990,
        lighting=led_lighting(12.0, 0.35, 0.7, 1.0)),

    "u08-bijeenkomst": project(
        "u08 Bijeenkomstgebouw 500 m² WP", "bijeenkomstfunctie", 500.0,
        envelope_utiliteit(500.0, storeys=1, glass_frac=0.30),
        heat_pump(3.8, "floor_heating"),
        {
            "generation": {"type": "heat_pump_dhw", "scop_dhw": 2.5},
            "emission": {"type": "utiliteit_kort"},
            "distribution_efficiency": 0.95,
            "shower_heat_recovery": None,
        },
        volume=500 * 4.5, year=2018, mass="light",
        vent={"system": {"type": "balanced", "wtw_efficiency": 0.78}},
        lighting=led_lighting(9.0, 0.25, 0.8, 0.95)),

    "u09-cellencomplex": project(
        "u09 Cellencomplex 350 m² gas", "celfunctie", 350.0,
        envelope_utiliteit(350.0, storeys=2, u_wall=0.35, glass_frac=0.10),
        gas_hr("hr104", "radiator_high_temp"),
        {
            "generation": {"type": "h_r_combi_boiler"},
            "emission": {"type": "utiliteit_lang"},
            "distribution_efficiency": 0.82,
            "shower_heat_recovery": None,
        },
        volume=350 * 3.0, year=2000,
        lighting=led_lighting(8.0, 0.80, 0.9, 1.0)),

    "u10-industriehal": project(
        "u10 Industriehal 1500 m² gas + absorptiekoeling", "industriefunctie",
        1500.0,
        envelope_utiliteit(1500.0, storeys=1, u_wall=0.45, u_roof=0.35,
                           glass_frac=0.05),
        gas_hr("hr100", "air_heating"),
        {
            "generation": {"type": "h_r_combi_boiler"},
            "emission": {"type": "utiliteit_kort"},
            "distribution_efficiency": 0.90,
            "shower_heat_recovery": None,
            # Industriefunctie heeft geen tabel-13.1-forfait: expliciet Q_W;nd
            "annual_demand_kwh": 2500.0,
        },
        volume=1500 * 8.0, year=1988,
        cooling={
            "system": {"type": "absorption_cooling", "cop": 0.7},
            "distribution": {"efficiency": 0.92},
            "emission": {"efficiency": 0.90, "regulation_factor": 0.92},
        },
        lighting=led_lighting(6.0, 0.40, 0.6, 1.0)),
}


def main():
    for name, data in FIXTURES.items():
        path = OUT / f"{name}.json"
        path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n",
                        encoding="utf-8")
        print(f"wrote {path.name}")
    print(f"\n{len(FIXTURES)} fixtures written to {OUT}")


if __name__ == "__main__":
    main()
