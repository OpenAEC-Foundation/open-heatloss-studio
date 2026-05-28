"""Vul RVO Rekentool Bijlage AA xlsm met een sample-case voor cross-validatie.

Output: een nieuwe xlsm in tests/references/ (gitignored) die user kan openen in
Excel met macros aan. Druk F9 → zie outputs in 'Projectgegevens en Resultaten' sheet.

Sample case 1: simpele 1-slaapkamer-woning, Zuid-georiënteerde voorgevel,
één raam in voorgevel, andere 3 gevels woningscheidend (geen buitenlucht), geen platdak.

Doel: minimale case waarmee onze Rust Bijlage AA engine 1-op-1 kan vergelijken.
"""
import shutil
import sys
from pathlib import Path

import openpyxl

sys.stdout.reconfigure(encoding="utf-8")

SRC = Path(r"C:/Github/open-heatloss-studio/tests/references/rekentool-bijlage-aa-nta8800-2025.04.xlsm")
DST = Path(r"C:/Github/open-heatloss-studio/tests/references/bijlage-aa-sample-case1-slaapkamer-zuid.xlsm")


def write_sample_case():
    shutil.copy(SRC, DST)
    wb = openpyxl.load_workbook(DST, keep_vba=True, data_only=False)

    # --- Projectgegevens en Resultaten ---
    pg = wb["Projectgegevens en Resultaten"]
    pg["B5"] = "Sample Case 1 - Single Bedroom Zuid"
    pg["B6"] = "SC1"
    pg["B7"] = "OpenAEC Foundation"
    pg["B8"] = "2026-05-27"
    pg["B9"] = "Cross-validatie open-heatloss-studio Bijlage AA engine"

    # BENG gegevens
    pg["B14"] = 2020  # Bouwjaar > 2015 → f_iso = 2.2
    pg["B15"] = "Nee"  # Niet nageisoleerd
    pg["B17"] = "Zuid"  # Orientatie voorzijde

    # Luchtstromen koeling (juli) - m3/h
    pg["B21"] = 5.0   # Infiltratie
    pg["B22"] = 0.0   # Natuurlijke toevoer
    pg["B23"] = 20.0  # Mechanische toevoer

    # Bezetting
    pg["B29"] = 12.0  # A_g rekenzone (= alleen Slaapkamer 1)
    pg["B30"] = 1     # 1 woonfunctie (default)

    # --- Ruimte 1: Slaapkamer Zuid ---
    r1 = wb["Ruimte 1"]
    r1["B3"] = "Slaapkamer 1"
    r1["B4"] = "Andere verblijfsruimte"  # niet woonvertrek
    r1["B5"] = 12.0  # Avr m²

    # Voorzijde (kolom B) — Zuid via gebouw-orientatie
    r1["B10"] = "Ja"   # Grenst aan buitenlucht
    r1["B11"] = 90     # Hellingshoek (gevel)
    r1["B13"] = 3.5    # Gevel lengte m
    r1["B14"] = 2.6    # Gevel hoogte m (→ oppervlak 9.1 m²)
    # Glasvlak type 1
    r1["B18"] = 2.0    # Aw m² (raamoppervlak inclusief kozijn)
    r1["B19"] = 1.2    # Uw W/m².K (HR++ glas + alu)
    r1["B20"] = 0.6    # g-waarde (zontoetredingsfactor)
    r1["B21"] = "Minimale belemmering"
    r1["B37"] = "Geen zonwering"
    # Glasvlak type 2 leeg (D18, D19 → niet gezet)

    # Linker zijde (kolom F), Achterzijde (J), Rechter zijde (N), Platdak (R)
    # → laten op "Nee" / 0 / default — geen wijziging nodig

    # Save
    wb.save(DST)
    print(f"OK: sample case geschreven naar {DST.name}")
    print(f"    grootte: {DST.stat().st_size} bytes")

    # Quick-check: lees terug
    wb2 = openpyxl.load_workbook(DST, keep_vba=True, data_only=False)
    pg2 = wb2["Projectgegevens en Resultaten"]
    r12 = wb2["Ruimte 1"]
    print(f"    Projectgegevens.B14 (Bouwjaar): {pg2['B14'].value}")
    print(f"    Projectgegevens.B17 (Orientatie voorzijde): {pg2['B17'].value}")
    print(f"    Projectgegevens.B29 (A_g): {pg2['B29'].value}")
    print(f"    Ruimte 1.B3 (Ruimtenaam): {r12['B3'].value}")
    print(f"    Ruimte 1.B5 (Avr): {r12['B5'].value}")
    print(f"    Ruimte 1.B10 (Grenst voorgevel): {r12['B10'].value}")
    print(f"    Ruimte 1.B18 (Glasvlak Aw): {r12['B18'].value}")
    print(f"    Ruimte 1.B19 (Uw): {r12['B19'].value}")
    print(f"    Ruimte 1.B20 (g-waarde): {r12['B20'].value}")


if __name__ == "__main__":
    write_sample_case()
