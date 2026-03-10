"""WFS Geo-data tool — haal PDOK data op en teken in Revit.

Perceelgrenzen, perceelnummers, straatnamen en huisnummers
via configureerbare WFS bronnen.
IronPython 2.7 compatible.
"""
from __future__ import print_function

import os
import clr

clr.AddReference("RevitAPI")
clr.AddReference("RevitAPIUI")
clr.AddReference("PresentationCore")
clr.AddReference("PresentationFramework")
clr.AddReference("WindowsBase")

from Autodesk.Revit.DB import Transaction  # noqa: E402
from Autodesk.Revit.UI import TaskDialog  # noqa: E402

from System.Windows import Window  # noqa: E402
from System.Windows.Controls import CheckBox  # noqa: E402
from System.Windows.Markup import XamlReader  # noqa: E402
from System.IO import StringReader  # noqa: E402
from System.Xml import XmlReader  # noqa: E402

from gis2bim.api.wfs import WFSClient  # noqa: E402
from gis2bim.api.wfs_layers import (  # noqa: E402
    PDOK_LAYERS,
    DEFAULT_SELECTED,
)
from gis2bim.revit.geometry import (  # noqa: E402
    get_project_location_rd,
    features_to_polylines,
    features_to_annotations,
    create_model_lines,
    create_text_notes,
)


# ---------------------------------------------------------------------------
# Globals
# ---------------------------------------------------------------------------

doc = __revit__.ActiveUIDocument.Document  # noqa: F821
uidoc = __revit__.ActiveUIDocument  # noqa: F821


# ---------------------------------------------------------------------------
# WPF Window
# ---------------------------------------------------------------------------

class WFSWindow(Window):
    """WPF window voor WFS geo-data ophalen."""

    def __init__(self):
        self._load_xaml()
        self._layer_checkboxes = {}
        self._init_layers()
        self._init_location()

    def _load_xaml(self):
        """Laad de XAML UI definitie."""
        xaml_path = os.path.join(
            os.path.dirname(__file__), "UI.xaml"
        )
        with open(xaml_path, "r") as f:
            xaml_text = f.read()

        reader = XmlReader.Create(StringReader(xaml_text))
        window = XamlReader.Load(reader)

        # Kopieer properties van geladen window
        self.Title = window.Title
        self.Width = window.Width
        self.Height = window.Height
        self.WindowStartupLocation = window.WindowStartupLocation
        self.ResizeMode = window.ResizeMode
        self.Background = window.Background
        self.Content = window.Content

        # UI elementen opzoeken
        self.tbRdX = self._find("tbRdX")
        self.tbRdY = self._find("tbRdY")
        self.tbLocationSource = self._find("tbLocationSource")
        self.spLayers = self._find("spLayers")
        self.cbBbox = self._find("cbBbox")
        self.tbStatus = self._find("tbStatus")
        self.btnExecute = self._find("btnExecute")
        self.btnCancel = self._find("btnCancel")

        # Event handlers koppelen
        self.btnExecute.Click += self.execute_click
        self.btnCancel.Click += self.cancel_click

    def _find(self, name):
        """Zoek een benoemd element in de XAML tree."""
        return self.Content.FindName(name)

    def _init_layers(self):
        """Vul layer checkboxes vanuit configuratie."""
        for key, layer in PDOK_LAYERS.items():
            cb = CheckBox()
            cb.Content = layer.name
            cb.Tag = key
            cb.IsChecked = key in DEFAULT_SELECTED
            cb.Margin = System.Windows.Thickness(0, 2, 0, 2)
            cb.FontSize = 11
            self.spLayers.Children.Add(cb)
            self._layer_checkboxes[key] = cb

    def _init_location(self):
        """Zoek en toon projectlocatie."""
        location = get_project_location_rd(doc)
        if location:
            self.tbRdX.Text = "{:.2f}".format(location[0])
            self.tbRdY.Text = "{:.2f}".format(location[1])
            self.tbLocationSource.Text = "Locatie gevonden in projectparameters"
            self._rd_x = location[0]
            self._rd_y = location[1]
        else:
            self.tbRdX.Text = ""
            self.tbRdY.Text = ""
            self.tbLocationSource.Text = (
                "Geen locatie gevonden. Stel GIS2BIM_RD_X/Y in "
                "of configureer het Survey Point."
            )
            self._rd_x = None
            self._rd_y = None
            self.btnExecute.IsEnabled = False

    def _get_selected_layers(self):
        """Geef geselecteerde layer keys terug."""
        selected = []
        for key, cb in self._layer_checkboxes.items():
            if cb.IsChecked:
                selected.append(key)
        return selected

    def _get_bbox_radius(self):
        """Haal bbox radius op uit dropdown (in meters)."""
        item = self.cbBbox.SelectedItem
        if item and item.Tag:
            return float(str(item.Tag))
        return 50.0

    def _set_status(self, text):
        """Update statusbalk."""
        self.tbStatus.Text = text

    def execute_click(self, sender, args):
        """Voer WFS requests uit en teken geometry."""
        selected = self._get_selected_layers()
        if not selected:
            self._set_status("Selecteer minimaal 1 layer.")
            return

        if self._rd_x is None or self._rd_y is None:
            self._set_status("Geen geldige locatie beschikbaar.")
            return

        radius = self._get_bbox_radius()
        client = WFSClient()
        bbox = client.compute_bbox(self._rd_x, self._rd_y, radius)

        self.btnExecute.IsEnabled = False
        self._set_status("Data ophalen van PDOK...")

        stats = {"lines": 0, "texts": 0, "errors": []}

        # Haal features op per layer
        all_features = {}
        for key in selected:
            layer = PDOK_LAYERS.get(key)
            if not layer:
                continue
            try:
                self._set_status(
                    "Ophalen: {0}...".format(layer.name)
                )
                features = client.get_features(layer, bbox)
                all_features[key] = features
                self._set_status(
                    "{0}: {1} features".format(
                        layer.name, len(features)
                    )
                )
            except Exception as ex:
                stats["errors"].append(
                    "{0}: {1}".format(layer.name, str(ex))
                )

        # Teken in Revit
        if all_features:
            self._set_status("Tekenen in Revit...")
            try:
                line_count, text_count = _draw_features(
                    doc, uidoc, all_features,
                    self._rd_x, self._rd_y,
                )
                stats["lines"] = line_count
                stats["texts"] = text_count
            except Exception as ex:
                stats["errors"].append(
                    "Tekenen mislukt: {0}".format(str(ex))
                )

        # Resultaat
        self.btnExecute.IsEnabled = True

        if stats["errors"]:
            error_text = "\n".join(stats["errors"])
            self._set_status(
                "Klaar met fouten. {0} lijnen, {1} labels.".format(
                    stats["lines"], stats["texts"]
                )
            )
            TaskDialog.Show(
                "WFS Waarschuwingen",
                error_text,
            )
        else:
            self._set_status(
                "Klaar! {0} lijnen, {1} labels getekend.".format(
                    stats["lines"], stats["texts"]
                )
            )

        self.Close()

    def cancel_click(self, sender, args):
        """Sluit het venster."""
        self.Close()


# ---------------------------------------------------------------------------
# Revit tekenfuncties
# ---------------------------------------------------------------------------

def _draw_features(doc, uidoc, all_features, origin_rd_x, origin_rd_y):
    """Teken alle opgehaalde features in Revit.

    Returns:
        Tuple (line_count, text_count).
    """
    total_lines = 0
    total_texts = 0
    active_view = doc.ActiveView

    t = Transaction(doc, "WFS Geo-data Importeren")
    t.Start()

    try:
        for key, features in all_features.items():
            layer = PDOK_LAYERS.get(key)
            if not layer:
                continue

            if layer.geometry_type == "polygon":
                polylines = features_to_polylines(
                    features, origin_rd_x, origin_rd_y
                )
                count = create_model_lines(doc, polylines)
                total_lines += count

            if layer.label_field:
                annotations = features_to_annotations(
                    features, layer.label_field,
                    origin_rd_x, origin_rd_y,
                )
                count = create_text_notes(
                    doc, active_view, annotations
                )
                total_texts += count

        t.Commit()
    except Exception:
        if t.HasStarted():
            t.RollBack()
        raise

    return total_lines, total_texts


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

import System  # noqa: E402

if __name__ == "__main__" or True:
    window = WFSWindow()
    window.ShowDialog()
