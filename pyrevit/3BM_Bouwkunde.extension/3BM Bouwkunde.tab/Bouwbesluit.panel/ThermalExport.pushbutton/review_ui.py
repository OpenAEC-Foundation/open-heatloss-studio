# -*- coding: utf-8 -*-
"""Review UI voor thermische schil data — WinForms DataGridView.

Toont scan-resultaten in een tabbed interface zodat de gebruiker
rooms kan classificeren, constructies kan excluden, en openings
kan reviewen voordat de JSON export wordt geschreven.

IronPython 2.7 — geen f-strings, geen type hints.
Volgt het WinForms patroon van WarmteverliesExport.
"""
import clr
clr.AddReference("System.Windows.Forms")
clr.AddReference("System.Drawing")

from System.Windows.Forms import (
    Form, FormBorderStyle, FormStartPosition,
    TabControl, TabPage, Panel, Label, Button,
    DataGridView, DataGridViewTextBoxColumn,
    DataGridViewCheckBoxColumn, DataGridViewComboBoxColumn,
    DataGridViewSelectionMode, DataGridViewAutoSizeColumnsMode,
    AnchorStyles, DockStyle, Padding, FlatStyle,
    DialogResult, MessageBox, MessageBoxButtons, MessageBoxIcon,
)
from System.Drawing import Point, Size, Color, Font, FontStyle, ContentAlignment


# =============================================================================
# 3BM Huisstijl kleuren
# =============================================================================
_CLR_VIOLET = Color.FromArgb(53, 14, 53)
_CLR_TEAL = Color.FromArgb(69, 182, 168)
_CLR_BG_DARK = Color.FromArgb(38, 38, 38)
_CLR_BG_PANEL = Color.FromArgb(48, 48, 48)
_CLR_TEXT_LIGHT = Color.FromArgb(230, 230, 230)
_CLR_TEXT_DIM = Color.FromArgb(160, 160, 160)
_CLR_GRID_BG = Color.FromArgb(55, 55, 55)
_CLR_GRID_ALT = Color.FromArgb(62, 62, 62)
_CLR_GRID_HEADER = Color.FromArgb(45, 45, 45)

# Room type opties
ROOM_TYPE_OPTIONS = [
    ("Verwarmd", "heated"),
    ("Onverwarmd", "unheated"),
    ("Buiten", "outside"),
    ("Grond", "ground"),
    ("Water", "water"),
]

ROOM_TYPE_LABEL_MAP = {v: k for k, v in ROOM_TYPE_OPTIONS}
ROOM_TYPE_VALUE_MAP = dict(ROOM_TYPE_OPTIONS)


class ThermalReviewForm(Form):
    """WinForms review dialog voor thermische schil data."""

    def __init__(self, scan_data):
        """Initialiseer de review form.

        Args:
            scan_data: dict met rooms, constructions, openings, open_connections
        """
        self.scan_data = scan_data
        self.result_data = None  # Wordt gezet bij export

        self._init_form()
        self._build_header()
        self._build_tabs()
        self._build_footer()

    def _init_form(self):
        """Initialiseer form eigenschappen."""
        self.Text = "Thermal Export Review"
        self.Size = Size(950, 700)
        self.MinimumSize = Size(800, 550)
        self.StartPosition = FormStartPosition.CenterScreen
        self.FormBorderStyle = FormBorderStyle.Sizable
        self.BackColor = _CLR_BG_DARK
        self.ForeColor = _CLR_TEXT_LIGHT
        self.Font = Font("Segoe UI", 9.0)

    def _build_header(self):
        """Bouw het header panel met titel."""
        pnl = Panel()
        pnl.Dock = DockStyle.Top
        pnl.Height = 60
        pnl.BackColor = _CLR_VIOLET
        pnl.Padding = Padding(15, 10, 15, 10)

        lbl_title = Label()
        lbl_title.Text = "Thermal Export Review"
        lbl_title.Font = Font("Segoe UI", 16.0, FontStyle.Bold)
        lbl_title.ForeColor = Color.White
        lbl_title.AutoSize = True
        lbl_title.Location = Point(15, 8)
        pnl.Controls.Add(lbl_title)

        lbl_sub = Label()
        lbl_sub.Text = "Controleer en pas de thermische schil data aan voor export"
        lbl_sub.Font = Font("Segoe UI", 9.0)
        lbl_sub.ForeColor = Color.FromArgb(200, 200, 200)
        lbl_sub.AutoSize = True
        lbl_sub.Location = Point(15, 35)
        pnl.Controls.Add(lbl_sub)

        self.Controls.Add(pnl)

    def _build_tabs(self):
        """Bouw het TabControl met 4 tabs."""
        self.tabs = TabControl()
        self.tabs.Dock = DockStyle.Fill
        self.tabs.Font = Font("Segoe UI", 9.0)
        self.tabs.Padding = Point(10, 5)

        self._build_tab_rooms()
        self._build_tab_constructions()
        self._build_tab_openings()
        self._build_tab_summary()

        self.Controls.Add(self.tabs)

        # Tab moet na header komen (z-order)
        self.tabs.BringToFront()

    def _build_footer(self):
        """Bouw het footer panel met knoppen."""
        pnl = Panel()
        pnl.Dock = DockStyle.Bottom
        pnl.Height = 55
        pnl.BackColor = _CLR_BG_PANEL
        pnl.Padding = Padding(15, 10, 15, 10)

        # Export knop
        btn_export = Button()
        btn_export.Text = "Exporteer JSON"
        btn_export.Size = Size(140, 35)
        btn_export.Location = Point(15, 10)
        btn_export.BackColor = _CLR_TEAL
        btn_export.ForeColor = Color.White
        btn_export.Font = Font("Segoe UI", 10.0, FontStyle.Bold)
        btn_export.FlatStyle = FlatStyle.Flat
        btn_export.FlatAppearance.BorderSize = 0
        btn_export.Click += self._on_export_click
        pnl.Controls.Add(btn_export)

        # Annuleer knop
        btn_cancel = Button()
        btn_cancel.Text = "Annuleren"
        btn_cancel.Size = Size(100, 35)
        btn_cancel.Location = Point(165, 10)
        btn_cancel.BackColor = Color.FromArgb(80, 80, 80)
        btn_cancel.ForeColor = _CLR_TEXT_LIGHT
        btn_cancel.Font = Font("Segoe UI", 10.0)
        btn_cancel.FlatStyle = FlatStyle.Flat
        btn_cancel.FlatAppearance.BorderSize = 0
        btn_cancel.Click += self._on_cancel_click
        pnl.Controls.Add(btn_cancel)

        self.Controls.Add(pnl)

    # -----------------------------------------------------------------
    # Tab 1: Ruimtes
    # -----------------------------------------------------------------
    def _build_tab_rooms(self):
        """Bouw de Ruimtes tab met DataGridView."""
        tab = TabPage("Ruimtes")
        tab.BackColor = _CLR_BG_DARK
        self.tabs.TabPages.Add(tab)

        # Info label
        lbl = Label()
        lbl.Text = "Pas het type aan per ruimte (verwarmd/onverwarmd/buiten/grond)."
        lbl.ForeColor = _CLR_TEXT_DIM
        lbl.Font = Font("Segoe UI", 8.5)
        lbl.Location = Point(10, 8)
        lbl.AutoSize = True
        tab.Controls.Add(lbl)

        # DataGridView
        self.grid_rooms = self._create_grid()
        self.grid_rooms.Location = Point(10, 30)
        self.grid_rooms.Size = Size(900, 480)
        self.grid_rooms.Anchor = (
            AnchorStyles.Top | AnchorStyles.Bottom
            | AnchorStyles.Left | AnchorStyles.Right
        )

        # Kolommen
        self._add_text_col(self.grid_rooms, "id", "ID", 70, True)
        self._add_text_col(self.grid_rooms, "name", "Naam", 180, True)

        # Type combobox kolom
        type_col = DataGridViewComboBoxColumn()
        type_col.Name = "type"
        type_col.HeaderText = "Type"
        type_col.Width = 120
        type_col.FlatStyle = FlatStyle.Flat
        for label, value in ROOM_TYPE_OPTIONS:
            type_col.Items.Add(label)
        self.grid_rooms.Columns.Add(type_col)

        self._add_text_col(self.grid_rooms, "level", "Niveau", 100, True)
        self._add_text_col(self.grid_rooms, "area", "Opp. [m2]", 80, True)
        self._add_text_col(self.grid_rooms, "height", "Hoogte [m]", 80, True)
        self._add_text_col(self.grid_rooms, "volume", "Volume [m3]", 90, True)

        # Vullen
        rooms = self.scan_data.get("rooms", [])
        for room in rooms:
            row_idx = self.grid_rooms.Rows.Add()
            row = self.grid_rooms.Rows[row_idx]
            row.Cells["id"].Value = room.get("id", "")
            row.Cells["name"].Value = room.get("name", "")

            rtype = room.get("type", "heated")
            type_label = ROOM_TYPE_LABEL_MAP.get(rtype, "Verwarmd")
            row.Cells["type"].Value = type_label

            row.Cells["level"].Value = room.get("level", "")
            row.Cells["area"].Value = "{0:.1f}".format(room.get("area_m2", 0))
            row.Cells["height"].Value = "{0:.2f}".format(room.get("height_m", 0))
            row.Cells["volume"].Value = "{0:.1f}".format(room.get("volume_m3", 0))

        tab.Controls.Add(self.grid_rooms)

    # -----------------------------------------------------------------
    # Tab 2: Constructies
    # -----------------------------------------------------------------
    def _build_tab_constructions(self):
        """Bouw de Constructies tab met DataGridView."""
        tab = TabPage("Constructies")
        tab.BackColor = _CLR_BG_DARK
        self.tabs.TabPages.Add(tab)

        lbl = Label()
        lbl.Text = "Vink constructies uit die je wilt excluden van de export."
        lbl.ForeColor = _CLR_TEXT_DIM
        lbl.Font = Font("Segoe UI", 8.5)
        lbl.Location = Point(10, 8)
        lbl.AutoSize = True
        tab.Controls.Add(lbl)

        self.grid_constr = self._create_grid()
        self.grid_constr.Location = Point(10, 30)
        self.grid_constr.Size = Size(900, 480)
        self.grid_constr.Anchor = (
            AnchorStyles.Top | AnchorStyles.Bottom
            | AnchorStyles.Left | AnchorStyles.Right
        )

        # Exclude checkbox
        chk_col = DataGridViewCheckBoxColumn()
        chk_col.Name = "exclude"
        chk_col.HeaderText = "Excl."
        chk_col.Width = 45
        chk_col.ReadOnly = False
        self.grid_constr.Columns.Add(chk_col)

        self._add_text_col(self.grid_constr, "id", "ID", 70, True)
        self._add_text_col(self.grid_constr, "room_a", "Ruimte A", 100, True)
        self._add_text_col(self.grid_constr, "room_b", "Ruimte B", 100, True)
        self._add_text_col(self.grid_constr, "orientation", "Orientatie", 80, True)
        self._add_text_col(self.grid_constr, "compass", "Kompas", 60, True)
        self._add_text_col(self.grid_constr, "area", "Opp. [m2]", 80, True)
        self._add_text_col(self.grid_constr, "layers", "Lagen", 50, True)
        self._add_text_col(self.grid_constr, "type_name", "Type", 180, True)

        # Room name lookup
        room_name_map = {}
        for r in self.scan_data.get("rooms", []):
            room_name_map[r["id"]] = r.get("name", r["id"])

        # Vullen
        constructions = self.scan_data.get("constructions", [])
        for constr in constructions:
            row_idx = self.grid_constr.Rows.Add()
            row = self.grid_constr.Rows[row_idx]
            row.Cells["exclude"].Value = False
            row.Cells["id"].Value = constr.get("id", "")

            ra_id = constr.get("room_a", "")
            rb_id = constr.get("room_b", "")
            row.Cells["room_a"].Value = room_name_map.get(ra_id, ra_id)
            row.Cells["room_b"].Value = room_name_map.get(rb_id, rb_id)

            row.Cells["orientation"].Value = constr.get("orientation", "")
            row.Cells["compass"].Value = constr.get("compass", "")
            row.Cells["area"].Value = "{0:.2f}".format(
                constr.get("gross_area_m2", 0)
            )
            row.Cells["layers"].Value = str(len(constr.get("layers", [])))
            row.Cells["type_name"].Value = constr.get("revit_type_name", "")

        tab.Controls.Add(self.grid_constr)

    # -----------------------------------------------------------------
    # Tab 3: Openingen
    # -----------------------------------------------------------------
    def _build_tab_openings(self):
        """Bouw de Openingen tab met DataGridView."""
        tab = TabPage("Openingen")
        tab.BackColor = _CLR_BG_DARK
        self.tabs.TabPages.Add(tab)

        lbl = Label()
        lbl.Text = "Vink openingen uit die je wilt excluden van de export."
        lbl.ForeColor = _CLR_TEXT_DIM
        lbl.Font = Font("Segoe UI", 8.5)
        lbl.Location = Point(10, 8)
        lbl.AutoSize = True
        tab.Controls.Add(lbl)

        self.grid_openings = self._create_grid()
        self.grid_openings.Location = Point(10, 30)
        self.grid_openings.Size = Size(900, 480)
        self.grid_openings.Anchor = (
            AnchorStyles.Top | AnchorStyles.Bottom
            | AnchorStyles.Left | AnchorStyles.Right
        )

        # Exclude checkbox
        chk_col = DataGridViewCheckBoxColumn()
        chk_col.Name = "exclude"
        chk_col.HeaderText = "Excl."
        chk_col.Width = 45
        chk_col.ReadOnly = False
        self.grid_openings.Columns.Add(chk_col)

        self._add_text_col(self.grid_openings, "id", "ID", 80, True)
        self._add_text_col(self.grid_openings, "constr", "Constructie", 90, True)
        self._add_text_col(self.grid_openings, "type", "Type", 80, True)
        self._add_text_col(self.grid_openings, "width", "Breedte [mm]", 90, True)
        self._add_text_col(self.grid_openings, "height", "Hoogte [mm]", 90, True)
        self._add_text_col(self.grid_openings, "type_name", "Revit Type", 220, True)

        # Vullen
        openings_data = self.scan_data.get("openings", [])
        for opening in openings_data:
            row_idx = self.grid_openings.Rows.Add()
            row = self.grid_openings.Rows[row_idx]
            row.Cells["exclude"].Value = False
            row.Cells["id"].Value = opening.get("id", "")
            row.Cells["constr"].Value = opening.get("construction_id", "")
            row.Cells["type"].Value = opening.get("type", "")
            row.Cells["width"].Value = "{0:.0f}".format(opening.get("width_mm", 0))
            row.Cells["height"].Value = "{0:.0f}".format(opening.get("height_mm", 0))
            row.Cells["type_name"].Value = opening.get("revit_type_name", "")

        tab.Controls.Add(self.grid_openings)

    # -----------------------------------------------------------------
    # Tab 4: Samenvatting
    # -----------------------------------------------------------------
    def _build_tab_summary(self):
        """Bouw de Samenvatting tab met totalen en waarschuwingen."""
        tab = TabPage("Samenvatting")
        tab.BackColor = _CLR_BG_DARK
        tab.Padding = Padding(15)
        self.tabs.TabPages.Add(tab)

        rooms = self.scan_data.get("rooms", [])
        constrs = self.scan_data.get("constructions", [])
        opens = self.scan_data.get("openings", [])

        heated = [r for r in rooms if r.get("type") == "heated"]
        unheated = [r for r in rooms if r.get("type") == "unheated"]
        pseudo = [r for r in rooms if r.get("type") in ("outside", "ground", "water")]

        total_constr_area = sum(c.get("gross_area_m2", 0) for c in constrs)
        constrs_with_layers = [c for c in constrs if c.get("layers")]
        constrs_no_layers = [c for c in constrs if not c.get("layers")]

        y = 10

        # Titel
        lbl_title = Label()
        lbl_title.Text = "Overzicht thermische schil"
        lbl_title.Font = Font("Segoe UI", 14.0, FontStyle.Bold)
        lbl_title.ForeColor = _CLR_TEAL
        lbl_title.AutoSize = True
        lbl_title.Location = Point(15, y)
        tab.Controls.Add(lbl_title)
        y += 35

        # Statistieken
        stats = [
            ("Verwarmde ruimtes:", str(len(heated))),
            ("Onverwarmde ruimtes:", str(len(unheated))),
            ("Pseudo-ruimtes (buiten/grond):", str(len(pseudo))),
            ("", ""),
            ("Constructies totaal:", str(len(constrs))),
            ("Constructies met laagopbouw:", str(len(constrs_with_layers))),
            ("Constructies zonder laagopbouw:", str(len(constrs_no_layers))),
            ("Totaal constructie-oppervlak:", "{0:.1f} m2".format(total_constr_area)),
            ("", ""),
            ("Openingen:", str(len(opens))),
        ]

        for label_text, value_text in stats:
            if not label_text:
                y += 10
                continue

            lbl_key = Label()
            lbl_key.Text = label_text
            lbl_key.Font = Font("Segoe UI", 10.0)
            lbl_key.ForeColor = _CLR_TEXT_LIGHT
            lbl_key.AutoSize = True
            lbl_key.Location = Point(15, y)
            tab.Controls.Add(lbl_key)

            lbl_val = Label()
            lbl_val.Text = value_text
            lbl_val.Font = Font("Segoe UI", 10.0, FontStyle.Bold)
            lbl_val.ForeColor = _CLR_TEAL
            lbl_val.AutoSize = True
            lbl_val.Location = Point(300, y)
            tab.Controls.Add(lbl_val)

            y += 25

        # Waarschuwingen
        warnings = []
        if len(constrs_no_layers) > 0:
            warnings.append(
                "{0} constructie(s) hebben geen laagopbouw — "
                "U-waarde moet handmatig worden ingevuld.".format(
                    len(constrs_no_layers)
                )
            )
        if len(heated) == 0:
            warnings.append("Geen verwarmde ruimtes gevonden!")

        rooms_no_height = [r for r in heated if r.get("height_m", 0) <= 0]
        if rooms_no_height:
            warnings.append(
                "{0} verwarmde ruimte(s) hebben geen hoogte.".format(
                    len(rooms_no_height)
                )
            )

        if warnings:
            y += 20
            lbl_warn_title = Label()
            lbl_warn_title.Text = "Waarschuwingen:"
            lbl_warn_title.Font = Font("Segoe UI", 11.0, FontStyle.Bold)
            lbl_warn_title.ForeColor = Color.FromArgb(255, 180, 50)
            lbl_warn_title.AutoSize = True
            lbl_warn_title.Location = Point(15, y)
            tab.Controls.Add(lbl_warn_title)
            y += 28

            for w in warnings:
                lbl_w = Label()
                lbl_w.Text = "  - {0}".format(w)
                lbl_w.Font = Font("Segoe UI", 9.0)
                lbl_w.ForeColor = Color.FromArgb(255, 200, 100)
                lbl_w.AutoSize = True
                lbl_w.Location = Point(15, y)
                tab.Controls.Add(lbl_w)
                y += 22

    # -----------------------------------------------------------------
    # Grid helpers
    # -----------------------------------------------------------------
    def _create_grid(self):
        """Maak een DataGridView met 3BM styling."""
        grid = DataGridView()
        grid.ReadOnly = False
        grid.AllowUserToAddRows = False
        grid.AllowUserToDeleteRows = False
        grid.SelectionMode = DataGridViewSelectionMode.FullRowSelect
        grid.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.None_
        grid.RowHeadersVisible = False
        grid.EnableHeadersVisualStyles = False
        grid.BorderStyle = 0  # None

        # Kleuren
        grid.BackgroundColor = _CLR_GRID_BG
        grid.DefaultCellStyle.BackColor = _CLR_GRID_BG
        grid.DefaultCellStyle.ForeColor = _CLR_TEXT_LIGHT
        grid.DefaultCellStyle.SelectionBackColor = _CLR_VIOLET
        grid.DefaultCellStyle.SelectionForeColor = Color.White
        grid.DefaultCellStyle.Font = Font("Segoe UI", 9.0)

        grid.AlternatingRowsDefaultCellStyle.BackColor = _CLR_GRID_ALT
        grid.AlternatingRowsDefaultCellStyle.ForeColor = _CLR_TEXT_LIGHT

        grid.ColumnHeadersDefaultCellStyle.BackColor = _CLR_GRID_HEADER
        grid.ColumnHeadersDefaultCellStyle.ForeColor = _CLR_TEAL
        grid.ColumnHeadersDefaultCellStyle.Font = Font(
            "Segoe UI", 9.0, FontStyle.Bold
        )
        grid.ColumnHeadersHeight = 30
        grid.RowTemplate.Height = 26

        return grid

    def _add_text_col(self, grid, name, header, width, readonly):
        """Voeg een tekst kolom toe aan een DataGridView."""
        col = DataGridViewTextBoxColumn()
        col.Name = name
        col.HeaderText = header
        col.Width = width
        col.ReadOnly = readonly
        grid.Columns.Add(col)

    # -----------------------------------------------------------------
    # Actions
    # -----------------------------------------------------------------
    def _on_export_click(self, sender, args):
        """Verzamel gefilterde data en sluit form."""
        self.result_data = self._collect_filtered_data()
        self.DialogResult = DialogResult.OK
        self.Close()

    def _on_cancel_click(self, sender, args):
        """Annuleer en sluit form."""
        self.result_data = None
        self.DialogResult = DialogResult.Cancel
        self.Close()

    def _collect_filtered_data(self):
        """Verzamel data met exclusies verwijderd en types bijgewerkt."""
        data = {
            "rooms": [],
            "constructions": [],
            "openings": [],
            "open_connections": self.scan_data.get("open_connections", []),
        }

        # Rooms: update types vanuit grid
        orig_rooms = self.scan_data.get("rooms", [])
        for row_idx in range(self.grid_rooms.Rows.Count):
            if row_idx >= len(orig_rooms):
                break
            room = dict(orig_rooms[row_idx])  # shallow copy
            type_label = self.grid_rooms.Rows[row_idx].Cells["type"].Value
            if type_label:
                type_value = ROOM_TYPE_VALUE_MAP.get(type_label, "heated")
                room["type"] = type_value
            data["rooms"].append(room)

        # Constructions: exclude verwijderen
        excluded_constr_ids = set()
        orig_constrs = self.scan_data.get("constructions", [])
        for row_idx in range(self.grid_constr.Rows.Count):
            if row_idx >= len(orig_constrs):
                break
            excluded = self.grid_constr.Rows[row_idx].Cells["exclude"].Value
            if excluded:
                excluded_constr_ids.add(orig_constrs[row_idx].get("id"))
            else:
                data["constructions"].append(orig_constrs[row_idx])

        # Openings: exclude verwijderen + filter op excluded constructions
        orig_openings = self.scan_data.get("openings", [])
        for row_idx in range(self.grid_openings.Rows.Count):
            if row_idx >= len(orig_openings):
                break
            excluded = self.grid_openings.Rows[row_idx].Cells["exclude"].Value
            if excluded:
                continue
            opening = orig_openings[row_idx]
            if opening.get("construction_id") in excluded_constr_ids:
                continue
            data["openings"].append(opening)

        return data

    def show_dialog(self):
        """Toon het dialog en retourneer gefilterde data of None.

        Returns:
            dict: Gefilterde scan data bij export, None bij annuleren
        """
        result = self.ShowDialog()
        if result == DialogResult.OK:
            return self.result_data
        return None
