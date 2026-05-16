# -*- coding: utf-8 -*-
"""Warmteverlies ISSO 51 Export — pyRevit pushbutton script.

Exporteert verwarmde ruimten en grensvlakken naar ISSO 51 JSON,
bruikbaar door de isso51-core engine en warmteverlies.open-aec.com.

IronPython 2.7 — geen f-strings, geen type hints.
"""

__title__ = "Warmteverlies\nExport"
__author__ = "3BM Bouwkunde"
__doc__ = "Exporteer rooms naar ISSO 51 warmteverlies JSON"

from pyrevit import revit, DB, forms, script

import clr
clr.AddReference("System.Windows.Forms")
clr.AddReference("System.Drawing")

from System.Windows.Forms import (
    TabControl, TabPage, ComboBox, ComboBoxStyle, TextBox,
    CheckBox, Label, Panel, GroupBox, DataGridView,
    DataGridViewTextBoxColumn, DataGridViewCheckBoxColumn,
    DataGridViewComboBoxColumn, DataGridViewSelectionMode,
    DataGridViewAutoSizeColumnsMode,
    AnchorStyles, DockStyle, Padding,
    DialogResult, SaveFileDialog, FlatStyle,
)
from System.Drawing import Point, Size, Color, Font, FontStyle

import os
import sys

# Lib imports (3BM shared lib + warmteverlies package)
sys.path.append(os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "lib"
))
from ui_template import BaseForm, UIFactory, DPIScaler, Huisstijl, LayoutHelper

from warmteverlies.room_collector import collect_rooms, generate_room_id
from warmteverlies.room_function_mapper import map_all_rooms
from warmteverlies.boundary_analyzer import analyze_boundaries
from warmteverlies.adjacent_detector import classify_boundaries
from warmteverlies.opening_extractor import extract_openings
from warmteverlies.uvalue_extractor import get_u_value
from warmteverlies.json_builder import build_project_json, export_to_file
from warmteverlies.constants import DEFAULT_ROOM_HEIGHT

# =============================================================================
# Configuratie mappings
# =============================================================================
BUILDING_TYPES = [
    ("Vrijstaand", "detached"),
    ("Twee-onder-een-kap", "semi_detached"),
    ("Tussenwoning", "terraced"),
    ("Hoekwoning", "end_of_terrace"),
    ("Portiekwoning", "porch"),
    ("Galerij", "gallery"),
    ("Gestapeld", "stacked"),
]

SECURITY_CLASSES = [
    ("A (cz = 0)", "a"),
    ("B (cz = 0.5)", "b"),
    ("C (cz = 1.0)", "c"),
]

VENT_SYSTEMS = [
    ("Systeem A (natuurlijk)", "system_a"),
    ("Systeem B (mech. toevoer)", "system_b"),
    ("Systeem C (mech. afvoer)", "system_c"),
    ("Systeem D (gebalanceerd)", "system_d"),
    ("Systeem E (vraaggestuurd)", "system_e"),
]

FROST_PROTECTIONS = [
    ("Onbekend", "unknown"),
    ("Centraal - bypass", "central_reduced_speed"),
    ("Centraal - enthalpie", "central_enthalpy"),
    ("Centraal - voorverwarming", "central_preheating"),
    ("Decentraal - bypass", "decentral_reduced_speed"),
    ("Decentraal - enthalpie", "decentral_enthalpy"),
    ("Decentraal - voorverwarming", "decentral_preheating"),
    ("Elektrisch - voorverwarming", "electric_preheating"),
]

INFILTRATION_METHODS = [
    ("Per buitenoppervlak", "per_exterior_area"),
    ("Per vloeroppervlak", "per_floor_area"),
]

FUNCTION_OPTIONS = [
    ("Woonkamer", "living_room"),
    ("Keuken", "kitchen"),
    ("Slaapkamer", "bedroom"),
    ("Badkamer", "bathroom"),
    ("Toilet", "toilet"),
    ("Hal/gang", "hallway"),
    ("Overloop", "landing"),
    ("Berging", "storage"),
    ("Zolder", "attic"),
    ("Overig", "custom"),
]


# =============================================================================
# Export Dialog — Windows Forms met BaseForm
# =============================================================================
class WarmteverliesExportForm(BaseForm):
    """Configuratiedialoog voor warmteverlies export."""

    def __init__(self, rooms):
        self.rooms = rooms
        self.export_config = None

        super(WarmteverliesExportForm, self).__init__(
            "Warmteverlies ISSO 51 Export", 850, 650
        )
        self.set_subtitle("Revit rooms naar ISSO 51 JSON")

        # Footer knoppen
        self.btn_export = self.add_footer_button(
            "Exporteer JSON", "primary", self._on_export_click, 150
        )

        self._build_ui()

    def _build_ui(self):
        """Bouw de tab-interface op in het content panel."""
        self.tabs = TabControl()
        self.tabs.Dock = DockStyle.Fill
        self.tabs.Font = Font("Segoe UI", UIFactory.FONT_NORMAL)
        self.pnl_content.Controls.Add(self.tabs)

        self._build_tab_gebouw()
        self._build_tab_klimaat()
        self._build_tab_ventilatie()
        self._build_tab_ruimten()

    # -----------------------------------------------------------------
    # Tab 1: Gebouw
    # -----------------------------------------------------------------
    def _build_tab_gebouw(self):
        tab = TabPage("Gebouw")
        tab.Padding = Padding(DPIScaler.scale(15))
        self.tabs.TabPages.Add(tab)

        y = 10
        lw = 180  # label width

        # Projectnaam
        lbl = UIFactory.create_label("Projectnaam")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_project_name = UIFactory.create_textbox(350)
        self.txt_project_name.Text = "Revit Export"
        self.txt_project_name.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_project_name)
        y += 38

        # Projectnummer
        lbl = UIFactory.create_label("Projectnummer")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_project_number = UIFactory.create_textbox(200)
        self.txt_project_number.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_project_number)
        y += 38

        # Gebouwtype
        lbl = UIFactory.create_label("Gebouwtype")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.cmb_building_type = UIFactory.create_combobox(
            250, [bt[0] for bt in BUILDING_TYPES]
        )
        self.cmb_building_type.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.cmb_building_type)
        y += 38

        # qv10
        lbl = UIFactory.create_label("qv10 [dm3/s]")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_qv10 = UIFactory.create_textbox(120)
        self.txt_qv10.Text = "150.0"
        self.txt_qv10.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_qv10)
        y += 38

        # Beveiligingsklasse
        lbl = UIFactory.create_label("Beveiligingsklasse")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.cmb_security = UIFactory.create_combobox(
            200, [sc[0] for sc in SECURITY_CLASSES]
        )
        self.cmb_security.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.cmb_security)
        y += 38

        # Aantal bouwlagen
        lbl = UIFactory.create_label("Aantal bouwlagen")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_num_floors = UIFactory.create_textbox(80)
        self.txt_num_floors.Text = "2"
        self.txt_num_floors.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_num_floors)
        y += 38

        # Gebouwhoogte
        lbl = UIFactory.create_label("Gebouwhoogte [m]")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_building_height = UIFactory.create_textbox(80)
        self.txt_building_height.Text = "6.0"
        self.txt_building_height.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_building_height)
        y += 38

        # Infiltratiemethode
        lbl = UIFactory.create_label("Infiltratiemethode")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.cmb_infiltration = UIFactory.create_combobox(
            250, [im[0] for im in INFILTRATION_METHODS]
        )
        self.cmb_infiltration.SelectedIndex = 1  # per_floor_area
        self.cmb_infiltration.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.cmb_infiltration)

    # -----------------------------------------------------------------
    # Tab 2: Klimaat
    # -----------------------------------------------------------------
    def _build_tab_klimaat(self):
        tab = TabPage("Klimaat")
        tab.Padding = Padding(DPIScaler.scale(15))
        self.tabs.TabPages.Add(tab)

        y = 10
        lw = 220

        # Buitentemperatuur
        lbl = UIFactory.create_label("Buitentemperatuur theta_e [C]")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_theta_e = UIFactory.create_textbox(80)
        self.txt_theta_e.Text = "-10.0"
        self.txt_theta_e.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_theta_e)
        y += 38

        # Grondtemperatuur
        lbl = UIFactory.create_label("Grondtemp. woning theta_b [C]")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_theta_b = UIFactory.create_textbox(80)
        self.txt_theta_b.Text = "17.0"
        self.txt_theta_b.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_theta_b)
        y += 38

        # Windfactor
        lbl = UIFactory.create_label("Windfactor [-]")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_wind_factor = UIFactory.create_textbox(80)
        self.txt_wind_factor.Text = "1.0"
        self.txt_wind_factor.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_wind_factor)

    # -----------------------------------------------------------------
    # Tab 3: Ventilatie
    # -----------------------------------------------------------------
    def _build_tab_ventilatie(self):
        tab = TabPage("Ventilatie")
        tab.Padding = Padding(DPIScaler.scale(15))
        self.tabs.TabPages.Add(tab)

        y = 10
        lw = 220

        # Systeem
        lbl = UIFactory.create_label("Ventilatiesysteem")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.cmb_vent_system = UIFactory.create_combobox(
            280, [vs[0] for vs in VENT_SYSTEMS]
        )
        self.cmb_vent_system.SelectedIndex = 2  # system_c
        self.cmb_vent_system.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.cmb_vent_system)
        y += 38

        # WTW
        self.chk_wtw = UIFactory.create_checkbox("WTW aanwezig", False)
        self.chk_wtw.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.chk_wtw)
        y += 38

        # WTW rendement
        lbl = UIFactory.create_label("WTW rendement [%]")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.txt_wtw_eff = UIFactory.create_textbox(80)
        self.txt_wtw_eff.Text = "80"
        self.txt_wtw_eff.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.txt_wtw_eff)
        y += 38

        # Vorstbeveiliging
        lbl = UIFactory.create_label("Vorstbeveiliging")
        lbl.Location = DPIScaler.scale_point(10, y + 4)
        tab.Controls.Add(lbl)
        self.cmb_frost = UIFactory.create_combobox(
            280, [fp[0] for fp in FROST_PROTECTIONS]
        )
        self.cmb_frost.Location = DPIScaler.scale_point(lw, y)
        tab.Controls.Add(self.cmb_frost)

    # -----------------------------------------------------------------
    # Tab 4: Ruimten
    # -----------------------------------------------------------------
    def _build_tab_ruimten(self):
        tab = TabPage("Ruimten")
        self.tabs.TabPages.Add(tab)

        # Instructie
        lbl_info = UIFactory.create_label(
            "Vink verwarmde ruimten aan en kies de juiste functie.",
            color=Huisstijl.TEXT_SECONDARY
        )
        lbl_info.Location = DPIScaler.scale_point(10, 8)
        tab.Controls.Add(lbl_info)

        # DataGridView
        columns = [
            ("number", "Nummer", 70),
            ("name", "Naam", 180),
            ("level", "Level", 100),
            ("area", "Opp. [m2]", 80),
            ("height", "Hoogte [m]", 80),
        ]
        self.grid_rooms = UIFactory.create_datagridview(
            columns, 750, 380, allow_edit=False
        )
        self.grid_rooms.Location = DPIScaler.scale_point(10, 35)
        self.grid_rooms.Anchor = (
            AnchorStyles.Top | AnchorStyles.Bottom
            | AnchorStyles.Left | AnchorStyles.Right
        )

        # Verwarmd checkbox kolom toevoegen
        chk_col = DataGridViewCheckBoxColumn()
        chk_col.Name = "heated"
        chk_col.HeaderText = "Verwarmd"
        chk_col.Width = DPIScaler.scale(65)
        chk_col.ReadOnly = False
        self.grid_rooms.Columns.Insert(0, chk_col)
        self.grid_rooms.ReadOnly = False

        # Functie combobox kolom toevoegen
        func_col = DataGridViewComboBoxColumn()
        func_col.Name = "function"
        func_col.HeaderText = "Functie"
        func_col.Width = DPIScaler.scale(130)
        func_col.FlatStyle = FlatStyle.Flat
        for label, value in FUNCTION_OPTIONS:
            func_col.Items.Add(label)
        self.grid_rooms.Columns.Add(func_col)

        # Vullen
        self._populate_rooms_grid()

        tab.Controls.Add(self.grid_rooms)

    def _populate_rooms_grid(self):
        """Vul de rooms DataGrid met room data."""
        func_label_map = {v: k for k, v in FUNCTION_OPTIONS}

        for room in self.rooms:
            row_idx = self.grid_rooms.Rows.Add()
            row = self.grid_rooms.Rows[row_idx]
            row.Cells["heated"].Value = room.get("is_heated", True)
            row.Cells["number"].Value = room.get("number", "")
            row.Cells["name"].Value = room.get("name", "")
            row.Cells["level"].Value = room.get("level_name", "")
            row.Cells["area"].Value = "{0:.1f}".format(
                room.get("floor_area_m2", 0)
            )
            row.Cells["height"].Value = "{0:.2f}".format(
                room.get("height_m", DEFAULT_ROOM_HEIGHT)
            )

            func = room.get("function", "custom")
            func_label = func_label_map.get(func, "Overig")
            row.Cells["function"].Value = func_label

    # -----------------------------------------------------------------
    # Actions
    # -----------------------------------------------------------------
    def _on_export_click(self, sender, args):
        """Verzamel configuratie en sluit form."""
        try:
            self.export_config = self._collect_config()
            self.Close()
        except Exception as ex:
            self.show_error(
                "Fout bij configuratie: {0}".format(str(ex))
            )

    def _collect_config(self):
        """Verzamel alle instellingen uit de UI controls."""
        config = {}

        # Gebouw tab
        config["project_name"] = self.txt_project_name.Text
        config["project_number"] = self.txt_project_number.Text
        config["building_type"] = BUILDING_TYPES[
            self.cmb_building_type.SelectedIndex
        ][1]
        config["qv10"] = _safe_float(self.txt_qv10.Text, 150.0)
        config["security_class"] = SECURITY_CLASSES[
            self.cmb_security.SelectedIndex
        ][1]
        config["num_floors"] = _safe_int(self.txt_num_floors.Text, 2)
        config["building_height"] = _safe_float(
            self.txt_building_height.Text, 6.0
        )
        config["infiltration_method"] = INFILTRATION_METHODS[
            self.cmb_infiltration.SelectedIndex
        ][1]

        # Klimaat tab
        config["theta_e"] = _safe_float(self.txt_theta_e.Text, -10.0)
        config["theta_b"] = _safe_float(self.txt_theta_b.Text, 17.0)
        config["wind_factor"] = _safe_float(self.txt_wind_factor.Text, 1.0)

        # Ventilatie tab
        config["ventilation_system"] = VENT_SYSTEMS[
            self.cmb_vent_system.SelectedIndex
        ][1]
        config["has_heat_recovery"] = self.chk_wtw.Checked
        config["heat_recovery_efficiency"] = (
            _safe_float(self.txt_wtw_eff.Text, 80.0) / 100.0
        )
        config["frost_protection"] = FROST_PROTECTIONS[
            self.cmb_frost.SelectedIndex
        ][1]

        # Rooms: update is_heated en function vanuit grid
        func_value_map = dict(FUNCTION_OPTIONS)
        for row_idx in range(self.grid_rooms.Rows.Count):
            row = self.grid_rooms.Rows[row_idx]
            is_heated = row.Cells["heated"].Value
            func_label = row.Cells["function"].Value or "Overig"
            func_value = func_value_map.get(func_label, "custom")

            if row_idx < len(self.rooms):
                self.rooms[row_idx]["is_heated"] = bool(is_heated)
                self.rooms[row_idx]["function"] = func_value

        # Totale vloeroppervlakte
        config["total_floor_area"] = sum(
            r["floor_area_m2"] for r in self.rooms if r.get("is_heated")
        )

        return config


# =============================================================================
# Helper functies
# =============================================================================
def _safe_float(text, default=0.0):
    """Veilige string naar float conversie."""
    try:
        return float(text.replace(",", "."))
    except (ValueError, AttributeError):
        return default


def _safe_int(text, default=0):
    """Veilige string naar int conversie."""
    try:
        return int(text)
    except (ValueError, AttributeError):
        return default


# =============================================================================
# Hoofdlogica
# =============================================================================
def run_export(doc):
    """Voer de volledige export workflow uit."""
    output = script.get_output()
    output.print_md("## Warmteverlies ISSO 51 Export")

    # Stap 1: Rooms verzamelen
    output.print_md("**Stap 1:** Rooms verzamelen...")
    rooms = collect_rooms(doc)
    if not rooms:
        forms.alert(
            "Geen rooms gevonden in het model.\n"
            "Plaats rooms via Architecture > Room.",
            title="Geen Rooms",
        )
        return

    output.print_md("Gevonden: **{0}** rooms".format(len(rooms)))

    # Stap 2: Functies mappen
    output.print_md("**Stap 2:** Functies toewijzen...")
    rooms = map_all_rooms(rooms)

    # Stap 3: Configuratiescherm
    dialog = WarmteverliesExportForm(rooms)
    dialog.ShowDialog()

    if dialog.export_config is None:
        output.print_md("*Export geannuleerd.*")
        return

    config = dialog.export_config
    rooms = dialog.rooms

    # Heated rooms filteren
    heated_rooms = [r for r in rooms if r.get("is_heated")]
    if not heated_rooms:
        forms.alert(
            "Geen verwarmde ruimten geselecteerd.",
            title="Geen Selectie",
        )
        return

    heated_room_ids = set(r["element_id"] for r in heated_rooms)
    output.print_md(
        "Verwarmde ruimten: **{0}** van {1}".format(
            len(heated_rooms), len(rooms)
        )
    )

    # Stap 4: IDs genereren
    output.print_md("**Stap 3:** Room IDs genereren...")
    for room in heated_rooms:
        room["id"] = generate_room_id(room, heated_rooms)

    # Stap 5: Per room boundaries analyseren
    output.print_md("**Stap 4:** Grensvlakken analyseren...")
    rooms_with_boundaries = []

    for room_data in heated_rooms:
        room_element = room_data["element"]

        boundaries = analyze_boundaries(doc, room_element)
        boundaries = classify_boundaries(
            doc, room_data, boundaries, rooms, heated_room_ids
        )

        for boundary in boundaries:
            host = boundary.get("host_element")
            pos = boundary.get("position_type", "wall")
            bt = boundary.get("boundary_type", "exterior")
            u_val, u_src = get_u_value(doc, host, pos, bt)
            boundary["u_value"] = u_val
            boundary["u_source"] = u_src

        # Openings per wall
        openings_by_wall = {}
        for boundary in boundaries:
            host = boundary.get("host_element")
            host_cat = boundary.get("host_category")
            host_id = boundary.get("host_element_id")

            if host_cat == "Wall" and host is not None:
                wall_openings = extract_openings(doc, host)
                if wall_openings:
                    openings_by_wall[host_id] = wall_openings

        room_data["boundaries"] = boundaries
        room_data["openings"] = openings_by_wall
        rooms_with_boundaries.append(room_data)

        output.print_md(
            "  - {0}: {1} grensvlakken".format(
                room_data["name"], len(boundaries)
            )
        )

    # Stap 6: JSON opbouwen
    output.print_md("**Stap 5:** JSON opbouwen...")
    project_json = build_project_json(config, rooms_with_boundaries)

    room_count = len(project_json.get("rooms", []))
    total_constructions = sum(
        len(r.get("constructions", []))
        for r in project_json.get("rooms", [])
    )
    output.print_md(
        "JSON: **{0}** rooms, **{1}** constructies".format(
            room_count, total_constructions
        )
    )

    # Stap 7: Opslaan
    dlg = SaveFileDialog()
    dlg.Filter = "JSON bestanden (*.json)|*.json"
    dlg.DefaultExt = ".json"
    dlg.FileName = "{0}_warmteverlies.json".format(
        config.get("project_name", "export").replace(" ", "_")
    )

    if dlg.ShowDialog() == DialogResult.OK:
        file_path = dlg.FileName
        export_to_file(project_json, file_path)
        output.print_md(
            "**Opgeslagen:** `{0}`".format(file_path)
        )
        output.print_md(
            "\nOpen dit bestand op "
            "[warmteverlies.open-aec.com](https://warmteverlies.open-aec.com) "
            "om de berekening uit te voeren."
        )
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
        run_export(doc)
