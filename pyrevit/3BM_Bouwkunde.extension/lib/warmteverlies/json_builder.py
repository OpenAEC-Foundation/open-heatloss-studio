# -*- coding: utf-8 -*-
"""Bouw complete isso51-core project JSON uit geextraheerde Revit data.

Output conform schemas/v1/project.schema.json, direct bruikbaar
door de isso51-core engine en de web-app.
"""
import json
import datetime

from warmteverlies.constants import (
    DEFAULT_THETA_E,
    DEFAULT_THETA_B_RESIDENTIAL,
    DEFAULT_THETA_B_NON_RESIDENTIAL,
    DEFAULT_WIND_FACTOR,
    DEFAULT_U_EQUIVALENT,
    DEFAULT_GROUND_WATER_FACTOR,
    DEFAULT_FG2,
)
from warmteverlies.room_function_mapper import get_design_temperature


def build_project_json(config, rooms_with_boundaries):
    """Bouw het volledige project JSON object.

    Args:
        config: dict met gebouw/klimaat/ventilatie instellingen
        rooms_with_boundaries: list[dict] verwarmde rooms met boundaries

    Returns:
        dict: Volledig isso51-core project JSON
    """
    project = {
        "info": _build_info(config),
        "building": _build_building(config),
        "climate": _build_climate(config),
        "ventilation": _build_ventilation(config),
        "rooms": [],
    }

    for room_data in rooms_with_boundaries:
        room_json = _build_room(room_data, config)
        project["rooms"].append(room_json)

    return project


def export_to_file(project_json, file_path):
    """Exporteer project JSON naar bestand."""
    with open(file_path, "w") as f:
        json.dump(project_json, f, indent=2, ensure_ascii=False)
    return file_path


def _build_info(config):
    """Bouw het info blok."""
    today = datetime.date.today().isoformat()
    return {
        "name": config.get("project_name", "Revit Export"),
        "project_number": config.get("project_number", ""),
        "date": today,
        "engineer": "pyRevit ISSO 51 Export",
        "notes": "Geexporteerd uit Revit via 3BM pyRevit warmteverlies plugin",
    }


def _build_building(config):
    """Bouw het building blok."""
    return {
        "building_type": config.get("building_type", "detached"),
        "qv10": config.get("qv10", 150.0),
        "total_floor_area": config.get("total_floor_area", 0.0),
        "security_class": config.get("security_class", "a"),
        "has_night_setback": False,
        "warmup_time": 0.0,
        "building_height": config.get("building_height", 6.0),
        "num_floors": config.get("num_floors", 2),
        "infiltration_method": config.get(
            "infiltration_method", "per_floor_area"
        ),
    }


def _build_climate(config):
    """Bouw het climate blok."""
    return {
        "theta_e": config.get("theta_e", DEFAULT_THETA_E),
        "theta_b_residential": config.get(
            "theta_b", DEFAULT_THETA_B_RESIDENTIAL
        ),
        "theta_b_non_residential": DEFAULT_THETA_B_NON_RESIDENTIAL,
        "wind_factor": config.get("wind_factor", DEFAULT_WIND_FACTOR),
    }


def _build_ventilation(config):
    """Bouw het ventilation blok."""
    vent = {
        "system_type": config.get("ventilation_system", "system_c"),
        "has_heat_recovery": config.get("has_heat_recovery", False),
    }

    if vent["has_heat_recovery"]:
        vent["heat_recovery_efficiency"] = config.get(
            "heat_recovery_efficiency", 0.80
        )
        vent["frost_protection"] = config.get(
            "frost_protection", "central_preheating"
        )
        vent["supply_temperature"] = config.get(
            "supply_temperature", 17.5
        )

    return vent


def _build_room(room_data, config):
    """Bouw een room JSON object inclusief constructions."""
    room_id = room_data.get("id", "0.00")
    function = room_data.get("function", "custom")
    design_temp = get_design_temperature(function)

    room = {
        "id": room_id,
        "name": room_data.get("name", "Onbekend"),
        "function": function,
        "floor_area": round(room_data.get("floor_area_m2", 0.0), 2),
        "height": round(room_data.get("height_m", 2.6), 2),
        "constructions": [],
        "heating_system": room_data.get("heating_system", "radiator_lt"),
        "ventilation_rate": 0.0,
        "has_mechanical_exhaust": _has_exhaust(config),
        "has_mechanical_supply": _has_supply(config),
        "fraction_outside_air": 1.0,
        "clamp_positive": True,
    }

    # Custom temperature alleen als afwijkend van function default
    custom_temp = room_data.get("custom_temperature")
    if custom_temp is not None and custom_temp != design_temp:
        room["custom_temperature"] = custom_temp
    elif function == "custom":
        room["custom_temperature"] = design_temp

    boundaries = room_data.get("boundaries", [])
    openings = room_data.get("openings", {})

    # Groepeer boundaries per type+positie
    grouped = _group_boundaries(boundaries)

    for key, group in grouped.items():
        boundary_type, position_type = key
        total_area = sum(b["area_m2"] for b in group)
        u_value = group[0].get("u_value", 0.21)

        # Openings aftrekken van wanden
        wall_openings = []
        if position_type == "wall":
            for b in group:
                host_id = b.get("host_element_id")
                if host_id and host_id in openings:
                    wall_openings.extend(openings[host_id])

        total_opening_area = sum(o["area_m2"] for o in wall_openings)
        net_area = max(0.0, total_area - total_opening_area)

        # Skip wanden met 0 m2 netto (volledig bedekt door openings)
        if net_area < 0.01 and position_type == "wall":
            # Openings nog wel toevoegen als aparte constructies
            if wall_openings:
                opening_groups = _group_openings(wall_openings)
                for o_key, o_group in opening_groups.items():
                    o_cat, o_u = o_key
                    o_area = sum(o["area_m2"] for o in o_group)
                    o_suffix = "ext-{0}".format(o_cat)
                    o_id = "{0}-{1}".format(room_id, o_suffix)

                    o_construction = {
                        "id": o_id,
                        "description": _opening_description(o_cat, o_group),
                        "area": round(o_area, 2),
                        "u_value": o_u,
                        "boundary_type": boundary_type,
                        "material_type": "non_masonry",
                        "vertical_position": "wall",
                        "use_forfaitaire_thermal_bridge": True,
                        "has_embedded_heating": False,
                    }
                    room["constructions"].append(o_construction)
            continue

        suffix = _make_suffix(boundary_type, position_type, group)
        c_id = "{0}-{1}".format(room_id, suffix)

        construction = _build_construction(
            c_id=c_id,
            description=_make_description(boundary_type, position_type, group),
            area=net_area,
            u_value=u_value,
            boundary_type=boundary_type,
            position_type=position_type,
            group=group,
        )
        room["constructions"].append(construction)

        # Openings als aparte ConstructionElements
        if wall_openings:
            opening_groups = _group_openings(wall_openings)
            for o_key, o_group in opening_groups.items():
                o_cat, o_u = o_key
                o_area = sum(o["area_m2"] for o in o_group)
                o_suffix = "ext-{0}".format(o_cat)
                o_id = "{0}-{1}".format(room_id, o_suffix)

                o_construction = {
                    "id": o_id,
                    "description": _opening_description(o_cat, o_group),
                    "area": round(o_area, 2),
                    "u_value": o_u,
                    "boundary_type": boundary_type,
                    "material_type": "non_masonry",
                    "vertical_position": "wall",
                    "use_forfaitaire_thermal_bridge": True,
                    "has_embedded_heating": False,
                }
                room["constructions"].append(o_construction)

    return room


def _build_construction(
    c_id, description, area, u_value, boundary_type, position_type, group
):
    """Bouw een enkel ConstructionElement."""
    construction = {
        "id": c_id,
        "description": description,
        "area": round(area, 2),
        "u_value": round(u_value, 3),
        "boundary_type": boundary_type,
        "material_type": "masonry",
        "vertical_position": position_type,
        "use_forfaitaire_thermal_bridge": True,
        "has_embedded_heating": False,
    }

    if boundary_type == "adjacent_room":
        adj_temp = group[0].get("adjacent_temperature")
        if adj_temp is not None:
            construction["adjacent_temperature"] = adj_temp

    if boundary_type == "unheated_space":
        tf = group[0].get("temperature_factor")
        if tf is not None:
            construction["temperature_factor"] = round(tf, 4)

    if boundary_type == "ground":
        construction["u_value"] = 0.0
        construction["ground_params"] = {
            "u_equivalent": round(
                u_value if u_value > 0 else DEFAULT_U_EQUIVALENT, 3
            ),
            "ground_water_factor": DEFAULT_GROUND_WATER_FACTOR,
            "fg2": DEFAULT_FG2,
        }

    return construction


def _group_boundaries(boundaries):
    """Groepeer boundaries per (boundary_type, position_type)."""
    groups = {}
    for b in boundaries:
        bt = b.get("boundary_type", "exterior")
        pt = b.get("position_type", "wall")
        adj_temp = b.get("adjacent_temperature")
        key = (bt, pt, adj_temp)

        if key not in groups:
            groups[key] = []
        groups[key].append(b)

    result = {}
    for key, group in groups.items():
        bt, pt, adj_temp = key
        simple_key = (bt, pt)
        if simple_key in result:
            if adj_temp is not None:
                diff_key = (bt + "-{0}".format(int(adj_temp)), pt)
            else:
                diff_key = (bt + "-extra", pt)
            result[diff_key] = group
        else:
            result[simple_key] = group

    return result


def _group_openings(openings):
    """Groepeer openings per (category, u_value)."""
    groups = {}
    for o in openings:
        key = (o["category"], round(o["u_value"], 2))
        if key not in groups:
            groups[key] = []
        groups[key].append(o)
    return groups


def _make_suffix(boundary_type, position_type, group):
    """Maak ID-suffix voor een construction element."""
    type_map = {
        "exterior": "ext",
        "adjacent_room": "adj",
        "unheated_space": "uh",
        "ground": "ground",
        "adjacent_building": "adj-bldg",
    }
    bt_prefix = type_map.get(boundary_type, "ext")

    pos_map = {
        "wall": "wall",
        "floor": "floor",
        "ceiling": "roof" if boundary_type == "exterior" else "ceil",
    }
    pos_suffix = pos_map.get(position_type, "wall")

    adj_temp = group[0].get("adjacent_temperature")
    if boundary_type == "adjacent_room" and adj_temp is not None:
        return "{0}-{1}-{2}".format(bt_prefix, int(adj_temp), pos_suffix)

    return "{0}-{1}".format(bt_prefix, pos_suffix)


def _make_description(boundary_type, position_type, group):
    """Maak beschrijving voor een construction element."""
    type_names = {
        "exterior": "Buiten",
        "adjacent_room": "Aangrenzend",
        "unheated_space": "Onverwarmd",
        "ground": "Grond",
        "adjacent_building": "Buurpand",
    }
    pos_names = {
        "wall": "wand",
        "floor": "vloer",
        "ceiling": "plafond/dak",
    }

    bt_name = type_names.get(boundary_type, boundary_type)
    pos_name = pos_names.get(position_type, position_type)

    adj_temp = group[0].get("adjacent_temperature")
    if boundary_type == "adjacent_room" and adj_temp is not None:
        return "{0}{1} naar {2} graden C ruimte".format(
            bt_name, pos_name, int(adj_temp)
        )

    return "{0}{1}".format(bt_name, pos_name)


def _opening_description(category, openings):
    """Maak beschrijving voor openings."""
    count = len(openings)
    if category == "window":
        return "Raam" if count == 1 else "Ramen ({0} stuks)".format(count)
    return "Deur" if count == 1 else "Deuren ({0} stuks)".format(count)


def _has_exhaust(config):
    """Bepaal of mechanische afzuiging aanwezig is."""
    system = config.get("ventilation_system", "system_c")
    return system in ("system_c", "system_d", "system_e")


def _has_supply(config):
    """Bepaal of mechanische toevoer aanwezig is."""
    system = config.get("ventilation_system", "system_c")
    return system in ("system_d", "system_e")
