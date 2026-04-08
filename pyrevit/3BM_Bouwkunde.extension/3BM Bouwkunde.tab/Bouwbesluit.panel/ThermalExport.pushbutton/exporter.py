# -*- coding: utf-8 -*-
"""JSON exporter voor thermische schil data.

Schrijft de scan-data naar een JSON bestand conform thermal-import schema v1.0.

IronPython 2.7 — geen f-strings, geen type hints.
"""
import json
import os
import datetime


def get_default_path(project_name):
    """Bepaal het standaard export pad in %TEMP%/3bm_exchange/.

    Args:
        project_name: Projectnaam voor bestandsnaam

    Returns:
        str: Volledig pad naar het JSON bestand
    """
    temp_dir = os.environ.get("TEMP", os.environ.get("TMP", "C:\\Temp"))
    exchange_dir = os.path.join(temp_dir, "3bm_exchange")

    if not os.path.exists(exchange_dir):
        os.makedirs(exchange_dir)

    safe_name = project_name.replace(" ", "_").replace("/", "_").replace("\\", "_")
    filename = "{0}_thermal.json".format(safe_name)
    return os.path.join(exchange_dir, filename)


def export_to_json(data, project_name, file_path):
    """Schrijf thermische schil data naar JSON bestand.

    Args:
        data: dict met rooms, constructions, openings, open_connections
        project_name: Naam van het project
        file_path: Volledig pad naar output bestand

    Returns:
        str: Het pad waarnaar geschreven is
    """
    # Metadata toevoegen
    output = {
        "version": "1.0",
        "source": "revit-eam",
        "exported_at": datetime.datetime.now().strftime("%Y-%m-%dT%H:%M:%S"),
        "project_name": project_name,
        "rooms": data.get("rooms", []),
        "constructions": data.get("constructions", []),
        "openings": data.get("openings", []),
        "open_connections": data.get("open_connections", []),
    }

    # Directory aanmaken indien nodig
    dir_path = os.path.dirname(file_path)
    if dir_path and not os.path.exists(dir_path):
        os.makedirs(dir_path)

    # Schrijf JSON met nette formatting
    with open(file_path, "w") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)

    return file_path
