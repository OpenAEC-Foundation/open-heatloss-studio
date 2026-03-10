"""PDOK WFS layer configuraties.

Voorgedefinieerde WFS layers voor Nederlandse geo-diensten (PDOK).
Uitbreidbaar met extra services (BGT, AHN, etc.).
"""
from gis2bim.api.wfs import WFSLayer


# ---------------------------------------------------------------------------
# PDOK Kadastrale Kaart WFS v5
# ---------------------------------------------------------------------------

KADASTER_WFS_URL = (
    "https://service.pdok.nl/kadaster/kadastralekaart/wfs/v5_0"
)

KADASTER_PERCELEN = WFSLayer(
    name="Perceelgrenzen",
    wfs_url=KADASTER_WFS_URL,
    layer_name="kadastralekaartv5:Perceel",
    geometry_type="polygon",
    label_field=None,
    crs="EPSG:28992",
)

KADASTER_PERCEELNUMMERS = WFSLayer(
    name="Perceelnummers",
    wfs_url=KADASTER_WFS_URL,
    layer_name="kadastralekaartv5:Perceelnummer",
    geometry_type="point",
    label_field="tekst",
    crs="EPSG:28992",
)

KADASTER_OPENBARERUIMTENAAM = WFSLayer(
    name="Straatnamen",
    wfs_url=KADASTER_WFS_URL,
    layer_name="kadastralekaartv5:OpenbareRuimteNaam",
    geometry_type="point",
    label_field="tekst",
    crs="EPSG:28992",
)

# ---------------------------------------------------------------------------
# PDOK BAG WFS
# ---------------------------------------------------------------------------

BAG_WFS_URL = (
    "https://service.pdok.nl/lv/bag/wfs/v2_0"
)

BAG_PAND = WFSLayer(
    name="BAG Panden",
    wfs_url=BAG_WFS_URL,
    layer_name="bag:pand",
    geometry_type="polygon",
    label_field=None,
    crs="EPSG:28992",
)

BAG_NUMMERAANDUIDING = WFSLayer(
    name="Huisnummers",
    wfs_url=BAG_WFS_URL,
    layer_name="bag:nummeraanduiding",
    geometry_type="point",
    label_field="huisnummer",
    crs="EPSG:28992",
)

# ---------------------------------------------------------------------------
# Gegroepeerde presets voor UI
# ---------------------------------------------------------------------------

PDOK_LAYERS = {
    "kadaster_percelen": KADASTER_PERCELEN,
    "kadaster_perceelnummers": KADASTER_PERCEELNUMMERS,
    "kadaster_straatnamen": KADASTER_OPENBARERUIMTENAAM,
    "bag_panden": BAG_PAND,
    "bag_huisnummers": BAG_NUMMERAANDUIDING,
}

# Standaard selectie bij openen tool
DEFAULT_SELECTED = [
    "kadaster_percelen",
    "kadaster_perceelnummers",
    "bag_huisnummers",
]
