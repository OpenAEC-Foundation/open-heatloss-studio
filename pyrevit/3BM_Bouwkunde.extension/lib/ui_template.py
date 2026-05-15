# -*- coding: utf-8 -*-
"""
ui_template.py — gedeelde WinForms UI-helpers voor pyRevit pushbuttons.

OpenAEC huisstijl (teal accent, lichte achtergrond), DPI-aware scaling
en factory-methods voor consistente controls. Werkt op IronPython 2.7 +
.NET Framework 4.8 (Revit 2025).

Exports:
    Huisstijl     — color/font tokens
    DPIScaler     — DPI-aware scaling helpers
    UIFactory     — control factory met consistente styling
    LayoutHelper  — kleine layout-hulpfuncties
    BaseForm      — Form-base met header (title + subtitle) en footer-buttons

Geleverd in deze repo zodat WarmteverliesExport.pushbutton (en analoge
toekomstige scripts) zonder externe 3BM-shared dependency draaien.
"""

import clr

clr.AddReference("System.Drawing")
clr.AddReference("System.Windows.Forms")

from System.Drawing import (
    Color,
    Font,
    FontStyle,
    Point,
    Size,
    SystemFonts,
)
from System.Windows.Forms import (
    AnchorStyles,
    BorderStyle,
    Button,
    CheckBox,
    ComboBox,
    ComboBoxStyle,
    DataGridView,
    DataGridViewSelectionMode,
    DialogResult,
    DockStyle,
    FlatStyle,
    Form,
    FormBorderStyle,
    FormStartPosition,
    Label,
    Padding,
    Panel,
    ScrollBars,
    TextBox,
)


# =============================================================================
# Huisstijl — OpenAEC kleuren + fonts
# =============================================================================
class Huisstijl(object):
    """Color tokens. Wijzig hier voor app-brede restyling."""

    # Primary palette (OpenAEC teal)
    PRIMARY = Color.FromArgb(15, 118, 110)      # #0F766E teal-700
    PRIMARY_HOVER = Color.FromArgb(13, 102, 96)  # darker
    ACCENT = Color.FromArgb(245, 158, 11)        # amber-500

    # Surface
    BG = Color.FromArgb(248, 250, 252)           # slate-50
    SURFACE = Color.White
    SURFACE_ALT = Color.FromArgb(241, 245, 249)  # slate-100

    # Text
    TEXT = Color.FromArgb(15, 23, 42)            # slate-900
    TEXT_SECONDARY = Color.FromArgb(71, 85, 105)  # slate-600
    TEXT_MUTED = Color.FromArgb(148, 163, 184)   # slate-400

    # Borders / status
    BORDER = Color.FromArgb(226, 232, 240)       # slate-200
    DANGER = Color.FromArgb(220, 38, 38)         # red-600
    SUCCESS = Color.FromArgb(22, 163, 74)        # green-600

    # Button on-color
    ON_PRIMARY = Color.White


# =============================================================================
# DPIScaler — scale ints + points met DPI-factor
# =============================================================================
class DPIScaler(object):
    """DPI-aware scaling. 1.0 op 96 DPI, 1.5 op 144 DPI, etc.

    Detecteert DPI lazy via een dummy Form bij eerste gebruik. Cached
    zodat herhaalde aanroepen goedkoop zijn.
    """

    _factor = None

    @classmethod
    def factor(cls):
        if cls._factor is None:
            try:
                # CreateGraphics geeft de DpiX/Y van het scherm waarop
                # de Form gezet wordt. 96 DPI = 1.0 schalingsfactor.
                f = Form()
                try:
                    g = f.CreateGraphics()
                    try:
                        cls._factor = float(g.DpiX) / 96.0
                    finally:
                        g.Dispose()
                finally:
                    f.Dispose()
            except Exception:
                cls._factor = 1.0
        return cls._factor

    @classmethod
    def scale(cls, value):
        """Scale an int by the DPI factor — returns int (System.Drawing
        APIs want ints, niet floats)."""
        return int(round(value * cls.factor()))

    @classmethod
    def scale_point(cls, x, y):
        return Point(cls.scale(x), cls.scale(y))

    @classmethod
    def scale_size(cls, w, h):
        return Size(cls.scale(w), cls.scale(h))


# =============================================================================
# UIFactory — control factory met consistente styling
# =============================================================================
class UIFactory(object):
    """Static factory voor WinForms controls met OpenAEC styling.

    Alle controls krijgen Segoe UI font, juiste kleuren en DPI-scaled
    sizes. Locations worden door de caller gezet (UIFactory positioneert
    niet — alleen layout-bouwer kent context).
    """

    FONT_NORMAL = 9.0
    FONT_SMALL = 8.0
    FONT_LARGE = 11.0

    @staticmethod
    def _font(size=None, bold=False):
        if size is None:
            size = UIFactory.FONT_NORMAL
        style = FontStyle.Bold if bold else FontStyle.Regular
        return Font("Segoe UI", size, style)

    @staticmethod
    def create_label(text, color=None, bold=False, font_size=None):
        lbl = Label()
        lbl.Text = text
        lbl.AutoSize = True
        lbl.Font = UIFactory._font(font_size, bold)
        lbl.ForeColor = color if color is not None else Huisstijl.TEXT
        lbl.BackColor = Color.Transparent
        return lbl

    @staticmethod
    def create_textbox(width, placeholder=None, multiline=False):
        tb = TextBox()
        tb.Size = DPIScaler.scale_size(width, 23 if not multiline else 60)
        tb.Font = UIFactory._font()
        tb.BorderStyle = BorderStyle.FixedSingle
        tb.BackColor = Huisstijl.SURFACE
        tb.ForeColor = Huisstijl.TEXT
        if multiline:
            tb.Multiline = True
            tb.ScrollBars = ScrollBars.Vertical
        if placeholder:
            # IronPython kent geen native PlaceholderText vóór .NET 5,
            # dus zet hem als Tag — caller kan eventueel een focus-handler
            # implementeren.
            tb.Tag = placeholder
        return tb

    @staticmethod
    def create_combobox(items=None, width=200):
        """items: list of (display_text, internal_value) tuples — alleen
        display_text wordt in de combo getoond; caller leest SelectedIndex
        en mapt zelf naar internal_value."""
        cb = ComboBox()
        cb.Size = DPIScaler.scale_size(width, 23)
        cb.Font = UIFactory._font()
        cb.DropDownStyle = ComboBoxStyle.DropDownList
        cb.FlatStyle = FlatStyle.Standard
        cb.BackColor = Huisstijl.SURFACE
        cb.ForeColor = Huisstijl.TEXT
        if items:
            for it in items:
                if isinstance(it, tuple):
                    cb.Items.Add(it[0])
                else:
                    cb.Items.Add(it)
            if cb.Items.Count > 0:
                cb.SelectedIndex = 0
        return cb

    @staticmethod
    def create_checkbox(text, checked=False):
        cb = CheckBox()
        cb.Text = text
        cb.AutoSize = True
        cb.Font = UIFactory._font()
        cb.Checked = bool(checked)
        cb.ForeColor = Huisstijl.TEXT
        cb.BackColor = Color.Transparent
        return cb

    @staticmethod
    def create_button(text, style="default", width=120, height=30, on_click=None):
        """style: "primary" (teal fill, white tekst) of "default" (light grey)."""
        btn = Button()
        btn.Text = text
        btn.Size = DPIScaler.scale_size(width, height)
        btn.Font = UIFactory._font(bold=(style == "primary"))
        btn.FlatStyle = FlatStyle.Flat
        btn.FlatAppearance.BorderSize = 1
        if style == "primary":
            btn.BackColor = Huisstijl.PRIMARY
            btn.ForeColor = Huisstijl.ON_PRIMARY
            btn.FlatAppearance.BorderColor = Huisstijl.PRIMARY
        elif style == "danger":
            btn.BackColor = Huisstijl.DANGER
            btn.ForeColor = Huisstijl.ON_PRIMARY
            btn.FlatAppearance.BorderColor = Huisstijl.DANGER
        else:
            btn.BackColor = Huisstijl.SURFACE_ALT
            btn.ForeColor = Huisstijl.TEXT
            btn.FlatAppearance.BorderColor = Huisstijl.BORDER
        btn.Cursor = None  # default cursor; system handles hover
        if on_click is not None:
            btn.Click += on_click
        return btn

    @staticmethod
    def create_datagridview(width=600, height=300):
        grid = DataGridView()
        grid.Size = DPIScaler.scale_size(width, height)
        grid.Font = UIFactory._font()
        grid.BackgroundColor = Huisstijl.SURFACE
        grid.GridColor = Huisstijl.BORDER
        grid.BorderStyle = BorderStyle.FixedSingle
        grid.RowHeadersVisible = False
        grid.AllowUserToAddRows = False
        grid.AllowUserToDeleteRows = False
        grid.AllowUserToResizeRows = False
        grid.SelectionMode = DataGridViewSelectionMode.FullRowSelect
        grid.MultiSelect = False
        # AutoSizeColumnsMode laten op default (None=1) — caller stelt
        # kolommen handmatig in via .Width. (DataGridViewAutoSizeColumnsMode.None
        # is een Python keyword-conflict in IronPython.)
        # Header styling
        try:
            grid.EnableHeadersVisualStyles = False
            grid.ColumnHeadersDefaultCellStyle.BackColor = Huisstijl.PRIMARY
            grid.ColumnHeadersDefaultCellStyle.ForeColor = Huisstijl.ON_PRIMARY
            grid.ColumnHeadersDefaultCellStyle.Font = UIFactory._font(bold=True)
            grid.ColumnHeadersDefaultCellStyle.SelectionBackColor = Huisstijl.PRIMARY
            grid.ColumnHeadersHeight = DPIScaler.scale(28)
        except Exception:
            pass
        # Alternating row colors
        try:
            grid.AlternatingRowsDefaultCellStyle.BackColor = Huisstijl.SURFACE_ALT
        except Exception:
            pass
        return grid


# =============================================================================
# LayoutHelper — kleine layout-utilities
# =============================================================================
class LayoutHelper(object):
    """Minimale helpers voor verticale stacks en row-spacing."""

    ROW_HEIGHT = 32        # standaard rij-hoogte (label + control)
    ROW_GAP = 4            # extra ruimte tussen rijen
    SECTION_GAP = 12       # ruimte tussen secties

    @staticmethod
    def next_row(y, height=None):
        """Geef de Y-coordinaat voor de volgende rij terug.

        Bij gebruik:
            y = 10
            y = LayoutHelper.next_row(y)   # 10 + ROW_HEIGHT
            y = LayoutHelper.next_row(y, height=60)  # custom rij-hoogte
        """
        h = height if height is not None else LayoutHelper.ROW_HEIGHT
        return y + h + LayoutHelper.ROW_GAP

    @staticmethod
    def section_break(y):
        return y + LayoutHelper.SECTION_GAP


# =============================================================================
# BaseForm — gedeelde Form-basis met header + content + footer
# =============================================================================
class BaseForm(Form):
    """Basis-Form met:
      - Header: titel + optionele subtitle, teal accent-balk eronder
      - Content: pnl_content (Panel) waarin de subclass z'n UI bouwt
      - Footer: knop-strip rechts onderaan (add_footer_button(...))
    """

    HEADER_H = 64
    FOOTER_H = 56
    ACCENT_H = 3

    def __init__(self, title, width=800, height=600):
        # IronPython 2 super() — Form is een .NET type, super werkt anders.
        # Direct __init__ van Form aanroepen is meest betrouwbaar.
        Form.__init__(self)
        self._title = title
        self.Text = title
        self.Size = DPIScaler.scale_size(width, height)
        self.MinimumSize = DPIScaler.scale_size(640, 480)
        self.StartPosition = FormStartPosition.CenterScreen
        self.FormBorderStyle = FormBorderStyle.Sizable
        self.BackColor = Huisstijl.BG
        self.Font = UIFactory._font()

        self._build_chrome()

    # -- chrome (header + footer) -------------------------------------------
    def _build_chrome(self):
        # Header panel
        self.pnl_header = Panel()
        self.pnl_header.Dock = DockStyle.Top
        self.pnl_header.Height = DPIScaler.scale(self.HEADER_H)
        self.pnl_header.BackColor = Huisstijl.SURFACE
        self.Controls.Add(self.pnl_header)

        # Title label
        self._lbl_title = Label()
        self._lbl_title.Text = self._title
        self._lbl_title.Font = UIFactory._font(size=14.0, bold=True)
        self._lbl_title.ForeColor = Huisstijl.PRIMARY
        self._lbl_title.AutoSize = True
        self._lbl_title.Location = DPIScaler.scale_point(20, 10)
        self.pnl_header.Controls.Add(self._lbl_title)

        # Subtitle label (initially empty)
        self._lbl_subtitle = Label()
        self._lbl_subtitle.Text = ""
        self._lbl_subtitle.Font = UIFactory._font(size=9.5)
        self._lbl_subtitle.ForeColor = Huisstijl.TEXT_SECONDARY
        self._lbl_subtitle.AutoSize = True
        self._lbl_subtitle.Location = DPIScaler.scale_point(20, 36)
        self.pnl_header.Controls.Add(self._lbl_subtitle)

        # Accent strip onder de header
        self.pnl_accent = Panel()
        self.pnl_accent.Dock = DockStyle.Top
        self.pnl_accent.Height = DPIScaler.scale(self.ACCENT_H)
        self.pnl_accent.BackColor = Huisstijl.PRIMARY
        self.Controls.Add(self.pnl_accent)

        # Footer panel (knoppen-strip)
        self.pnl_footer = Panel()
        self.pnl_footer.Dock = DockStyle.Bottom
        self.pnl_footer.Height = DPIScaler.scale(self.FOOTER_H)
        self.pnl_footer.BackColor = Huisstijl.SURFACE_ALT
        self._footer_buttons = []
        self.Controls.Add(self.pnl_footer)

        # Content panel — vult de resterende ruimte
        self.pnl_content = Panel()
        self.pnl_content.Dock = DockStyle.Fill
        self.pnl_content.BackColor = Huisstijl.BG
        self.pnl_content.Padding = Padding(DPIScaler.scale(10))
        self.Controls.Add(self.pnl_content)

        # Z-order: Dock-volgorde bepaalt layout; .Add zet onderaan in
        # z-order. WinForms Docks bottom + top + fill werken al correct
        # zolang Fill-control LAATST wordt toegevoegd — wat hier zo is.

    # -- public API ----------------------------------------------------------
    def set_subtitle(self, text):
        """Update de subtitle (kleinere text onder de title)."""
        self._lbl_subtitle.Text = text or ""

    def add_footer_button(self, label, style="default", on_click=None, width=120):
        """Voeg een knop toe aan de footer (rechts uitgelijnd, RTL stacked).

        style: "primary" / "default" / "danger" — zie UIFactory.create_button.
        Returns het Button-object zodat de caller hem eventueel later
        opnieuw kan stylen / Enabled toggelen.
        """
        btn = UIFactory.create_button(label, style=style, width=width, height=32, on_click=on_click)
        # Anker rechts-onder zodat hij meegroeit bij resize
        btn.Anchor = AnchorStyles.Top | AnchorStyles.Right
        # Bereken x-positie: stack from right
        right_padding = DPIScaler.scale(20)
        gap = DPIScaler.scale(8)
        used_width = sum(b.Width + gap for b in self._footer_buttons)
        x = self.pnl_footer.Width - right_padding - btn.Width - used_width
        y = DPIScaler.scale(12)
        btn.Location = Point(x, y)
        self._footer_buttons.append(btn)
        self.pnl_footer.Controls.Add(btn)

        # Re-layout existing buttons bij iedere toevoeging zodat ze van
        # rechts naar links netjes blijven stappen.
        self._relayout_footer_buttons()
        return btn

    def _relayout_footer_buttons(self):
        right_padding = DPIScaler.scale(20)
        gap = DPIScaler.scale(8)
        x_cursor = self.pnl_footer.Width - right_padding
        # Re-order vanaf rechts (laatst toegevoegde knop staat rechts? nee,
        # conventie: eerste toegevoegde knop is primary action rechts)
        for btn in self._footer_buttons:
            x_cursor -= btn.Width
            btn.Location = Point(x_cursor, DPIScaler.scale(12))
            x_cursor -= gap


__all__ = [
    "Huisstijl",
    "DPIScaler",
    "UIFactory",
    "LayoutHelper",
    "BaseForm",
]
