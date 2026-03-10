"""Generieke WFS (Web Feature Service) client.

Haalt GeoJSON features op van een WFS endpoint met bbox filter.
Compatibel met IronPython 2.7 (.NET WebClient voor HTTPS).
"""
from __future__ import print_function

import json

import clr
clr.AddReference("System")
from System.Net import WebClient  # noqa: E402
from System.Text import Encoding  # noqa: E402


class WFSLayer:
    """Configuratie voor een WFS layer."""

    def __init__(
        self,
        name,
        wfs_url,
        layer_name,
        geometry_type="polygon",
        label_field=None,
        crs="EPSG:28992",
        version="2.0.0",
        max_features=5000,
    ):
        self.name = name
        self.wfs_url = wfs_url
        self.layer_name = layer_name
        self.geometry_type = geometry_type
        self.label_field = label_field
        self.crs = crs
        self.version = version
        self.max_features = max_features


class Feature:
    """Een geo-feature met geometry en properties."""

    def __init__(self, geometry_type, coordinates, properties):
        self.geometry_type = geometry_type
        self.coordinates = coordinates
        self.properties = properties

    def get_label(self, field_name):
        """Haal label waarde op uit properties."""
        if not field_name or not self.properties:
            return None
        return self.properties.get(field_name)


class WFSClient:
    """Generieke WFS client die GeoJSON features ophaalt."""

    TIMEOUT_MS = 15000

    def __init__(self):
        self._client = WebClient()
        self._client.Encoding = Encoding.UTF8

    def get_features(self, layer, bbox):
        """Haal features op voor een layer binnen een bounding box.

        Args:
            layer: WFSLayer configuratie.
            bbox: Tuple (min_x, min_y, max_x, max_y) in layer CRS.

        Returns:
            Lijst van Feature objecten.
        """
        url = self._build_url(layer, bbox)
        response_text = self._fetch(url)
        return self._parse_geojson(response_text)

    def _build_url(self, layer, bbox):
        """Bouw WFS GetFeature URL met bbox filter."""
        bbox_str = "{0},{1},{2},{3}".format(
            bbox[0], bbox[1], bbox[2], bbox[3]
        )

        params = [
            "service=WFS",
            "version={0}".format(layer.version),
            "request=GetFeature",
            "typeName={0}".format(layer.layer_name),
            "outputFormat=application/json",
            "srsName={0}".format(layer.crs),
            "bbox={0},{1}".format(bbox_str, layer.crs),
            "count={0}".format(layer.max_features),
        ]

        separator = "&" if "?" in layer.wfs_url else "?"
        return layer.wfs_url + separator + "&".join(params)

    def _fetch(self, url):
        """HTTP GET request via .NET WebClient."""
        try:
            return self._client.DownloadString(url)
        except Exception as ex:
            raise RuntimeError(
                "WFS request mislukt: {0}".format(str(ex))
            )

    def _parse_geojson(self, text):
        """Parse GeoJSON FeatureCollection naar Feature lijst."""
        try:
            data = json.loads(text)
        except ValueError as ex:
            raise RuntimeError(
                "Ongeldig JSON antwoord van WFS: {0}".format(str(ex))
            )

        if data.get("type") != "FeatureCollection":
            raise RuntimeError(
                "Verwacht FeatureCollection, kreeg: {0}".format(
                    data.get("type", "onbekend")
                )
            )

        features = []
        for f in data.get("features", []):
            geom = f.get("geometry")
            if not geom:
                continue

            geom_type = geom.get("type", "")
            coords = geom.get("coordinates", [])
            props = f.get("properties", {})

            features.append(Feature(geom_type, coords, props))

        return features

    def compute_bbox(self, center_x, center_y, radius):
        """Bereken bounding box rond een punt.

        Args:
            center_x: X coordinaat van centrum (in CRS eenheden).
            center_y: Y coordinaat van centrum.
            radius: Halve zijde van de bbox in meters.

        Returns:
            Tuple (min_x, min_y, max_x, max_y).
        """
        return (
            center_x - radius,
            center_y - radius,
            center_x + radius,
            center_y + radius,
        )
