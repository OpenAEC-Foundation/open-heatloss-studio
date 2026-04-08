# -*- coding: utf-8 -*-
"""Thermal Export — pyRevit pushbutton script.

Exporteert de thermische schil van het Revit model via de
EnergyAnalysisDetailModel API naar JSON conform thermal-import v1.0,
bruikbaar door de warmteverlies import pipeline.

IronPython 2.7 — geen f-strings, geen type hints.
"""

__title__ = "Thermal\nExport"
__author__ = "3BM Bouwkunde"
__doc__ = "Exporteer thermische schil (EAM) naar JSON voor warmteverlies import"

from pyrevit import revit, DB, forms, script

import clr
clr.AddReference("System.Windows.Forms")
from System.Windows.Forms import DialogResult, SaveFileDialog

import os
import sys

# Local imports
_script_dir = os.path.dirname(__file__)
if _script_dir not in sys.path:
    sys.path.insert(0, _script_dir)

from eam_scanner import scan_thermal_shell
from exporter import export_to_json, get_default_path
from review_ui import ThermalReviewForm


# =============================================================================
# View validatie
# =============================================================================
def _check_view_visibility(doc, output):
    """Controleer of Rooms en Walls zichtbaar zijn in de actieve view.

    Geeft waarschuwingen bij verborgen categorien.
    """
    view = doc.ActiveView
    if view is None:
        return

    warnings = []
    checks = [
        (DB.BuiltInCategory.OST_Rooms, "Rooms"),
        (DB.BuiltInCategory.OST_Walls, "Walls"),
        (DB.BuiltInCategory.OST_Floors, "Floors"),
        (DB.BuiltInCategory.OST_Roofs, "Roofs"),
    ]

    for cat_enum, cat_name in checks:
        try:
            cat = DB.Category.GetCategory(doc, cat_enum)
            if cat is not None:
                vis = view.GetCategoryHidden(cat.Id)
                if vis:
                    warnings.append(cat_name)
        except Exception:
            pass

    if warnings:
        msg = (
            "De volgende categorieen zijn verborgen in de actieve view:\n"
            "{0}\n\n"
            "De EAM scanner werkt modelbreed, maar verborgen elementen "
            "kunnen wijzen op onvolledig gemodelleerde zones.\n\n"
            "Wil je doorgaan?"
        ).format(", ".join(warnings))

        if not forms.alert(msg, title="View Waarschuwing", yes=True, no=True):
            return False

    return True


# =============================================================================
# Hoofdlogica
# =============================================================================
def run_thermal_export(doc):
    """Voer de volledige thermal export workflow uit."""
    output = script.get_output()
    output.print_md("## Thermal Export (EAM)")
    output.print_md("*Thermische schil export via EnergyAnalysisDetailModel*")
    output.print_md("---")

    # Stap 1: View check
    output.print_md("**Stap 1:** View validatie...")
    result = _check_view_visibility(doc, output)
    if result is False:
        output.print_md("*Export geannuleerd door gebruiker.*")
        return

    # Stap 2: EAM scan
    output.print_md("**Stap 2:** EnergyAnalysisDetailModel scannen...")
    output.print_md("*Dit kan even duren afhankelijk van modelgrootte...*")

    scan_data = scan_thermal_shell(doc, output)

    if scan_data is None:
        forms.alert(
            "De EAM scan is mislukt.\n"
            "Controleer of het model rooms en wanden bevat.",
            title="Scan Mislukt",
        )
        return

    rooms = scan_data.get("rooms", [])
    constrs = scan_data.get("constructions", [])
    opens = scan_data.get("openings", [])

    if not rooms:
        forms.alert(
            "Geen ruimtes gevonden in het energy model.\n"
            "Controleer of het model rooms bevat.",
            title="Geen Ruimtes",
        )
        return

    output.print_md(
        "Scan resultaat: **{0}** ruimtes, **{1}** constructies, "
        "**{2}** openingen".format(len(rooms), len(constrs), len(opens))
    )

    # Stap 3: Review UI
    output.print_md("**Stap 3:** Review dialoog tonen...")

    review_form = ThermalReviewForm(scan_data)
    filtered_data = review_form.show_dialog()

    if filtered_data is None:
        output.print_md("*Export geannuleerd door gebruiker.*")
        return

    # Statistieken na review
    final_rooms = len(filtered_data.get("rooms", []))
    final_constrs = len(filtered_data.get("constructions", []))
    final_opens = len(filtered_data.get("openings", []))

    output.print_md(
        "Na review: **{0}** ruimtes, **{1}** constructies, "
        "**{2}** openingen".format(final_rooms, final_constrs, final_opens)
    )

    # Stap 4: Opslaan
    output.print_md("**Stap 4:** JSON opslaan...")

    # Projectnaam bepalen
    project_name = "Revit Export"
    try:
        title_param = doc.ProjectInformation.get_Parameter(
            DB.BuiltInParameter.PROJECT_NAME
        )
        if title_param and title_param.HasValue and title_param.AsString():
            project_name = title_param.AsString()
    except Exception:
        pass

    # SaveFileDialog
    dlg = SaveFileDialog()
    dlg.Filter = "JSON bestanden (*.json)|*.json"
    dlg.DefaultExt = ".json"
    dlg.FileName = "{0}_thermal.json".format(
        project_name.replace(" ", "_")
    )

    # Default pad instellen
    default_path = get_default_path(project_name)
    default_dir = os.path.dirname(default_path)
    if os.path.exists(default_dir):
        dlg.InitialDirectory = default_dir

    if dlg.ShowDialog() == DialogResult.OK:
        file_path = dlg.FileName

        try:
            export_to_json(filtered_data, project_name, file_path)
            output.print_md("---")
            output.print_md("**Opgeslagen:** `{0}`".format(file_path))
            output.print_md(
                "\nImporteer dit bestand in "
                "[warmteverlies.open-aec.com](https://warmteverlies.open-aec.com) "
                "via **Bestand > Importeer thermische schil**."
            )
        except Exception as ex:
            forms.alert(
                "Fout bij opslaan:\n{0}".format(str(ex)),
                title="Export Fout",
            )
            output.print_md("**FOUT:** {0}".format(str(ex)))
    else:
        output.print_md("*Opslaan geannuleerd.*")


# =============================================================================
# Entry point
# =============================================================================
if __name__ == "__main__":
    doc = revit.doc
    if doc is None:
        forms.alert("Geen Revit document geopend.", title="Fout")
    else:
        run_thermal_export(doc)
