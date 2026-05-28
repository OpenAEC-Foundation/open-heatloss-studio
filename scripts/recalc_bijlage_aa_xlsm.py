"""Start Excel via COM, open de bijlage AA xlsm, trigger CalculateFullRebuild,
lees de 11 uitlees-cellen + extra context. Save zodat openpyxl de cached values
in de toekomst ook kan zien.

Vereist: pywin32 + Microsoft Excel geinstalleerd.
"""

import io
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

import os
import shutil
import tempfile
from pathlib import Path

import win32com.client

XLSM_SRC = Path(__file__).parent.parent / "tests" / "references" / "bijlage-aa-sample-case1-slaapkamer-zuid.xlsm"
# Copy naar %TEMP% — Trust Center weigert vaak macro-xlsm uit git-repo paths
TMP_DIR = Path(tempfile.gettempdir())
XLSM_TMP = TMP_DIR / "bijlage-aa-sample-case1-recalc.xlsm"
shutil.copy(XLSM_SRC, XLSM_TMP)
XLSM_ABS = str(XLSM_TMP.resolve())
print(f"Working copy: {XLSM_ABS}")

# msoAutomationSecurityLow = 1 → enable macros zonder prompt
MSO_AUTOMATION_SECURITY_LOW = 1

excel = win32com.client.DispatchEx("Excel.Application")
excel.Visible = True  # User kan eventuele security-prompts interactief beantwoorden
excel.DisplayAlerts = True
excel.AutomationSecurity = MSO_AUTOMATION_SECURITY_LOW
print("Excel started visible — beantwoord eventuele macro-prompts handmatig...")
import time
time.sleep(2)

try:
    # Minimale COM-call zonder named args (sommige Excel-builds zijn picky)
    wb = excel.Workbooks.Open(XLSM_ABS)
    # Force full rebuild (alle formules + UDF's herrekenen)
    excel.CalculateFullRebuild()

    def read(sheet, coord, label):
        val = wb.Worksheets(sheet).Range(coord).Value
        print(f"  {coord:6s} ({label:55s}) = {val!r}")
        return val

    print("=== Ruimte 1 — uitlees-cellen ===")
    r1 = {}
    r1["B53"] = read("Ruimte 1", "B53", "θ_e max [°C]")
    r1["B55"] = read("Ruimte 1", "B55", "P_tr;ntr Voorgevel [W]")
    r1["B56"] = read("Ruimte 1", "B56", "P_sol Voorgevel [W]")
    r1["B57"] = read("Ruimte 1", "B57", "P_tr;gl Voorgevel [W]")
    r1["B58"] = read("Ruimte 1", "B58", "Totaal per gevel Voorgevel [W]")
    r1["B60"] = read("Ruimte 1", "B60", "P_int;calc [W]")
    r1["B61"] = read("Ruimte 1", "B61", "P_v;calc [W]")
    r1["B63"] = read("Ruimte 1", "B63", "Totaal koellastbijdrage [W]")
    r1["B64"] = read("Ruimte 1", "B64", "Koelbehoefte verblijfsruimte [W/m²]")

    print("\n=== Projectgegevens en Resultaten — uitlees-cellen ===")
    pr = {}
    pr["B16"] = read("Projectgegevens en Resultaten", "B16", "f;iso uit bouwjaarklasse [W/m²]")
    pr["B33"] = read("Projectgegevens en Resultaten", "B33", "q_int;calc;zi [W/m²]")
    pr["B49"] = read("Projectgegevens en Resultaten", "B49", "Koelbehoefte verblijfsruimte [W/m²]")
    pr["B51"] = read("Projectgegevens en Resultaten", "B51", "Min benodigde koelcap [W/m²]")
    pr["B55"] = read("Projectgegevens en Resultaten", "B55", "Benodigde koelcap Ruimte 1 [W]")
    pr["B68"] = read("Projectgegevens en Resultaten", "B68", "Min koelvermogen opwekker [W]")

    print("\n=== Extra context (input echo) ===")
    read("Projectgegevens en Resultaten", "B14", "Bouwjaar")
    read("Projectgegevens en Resultaten", "B17", "Orientatie voorzijde")
    read("Projectgegevens en Resultaten", "B18", "Orientatiehoek Ƴ [°]")
    read("Projectgegevens en Resultaten", "B29", "A_g [m²]")
    read("Projectgegevens en Resultaten", "B30", "Aantal woonfuncties")
    read("Projectgegevens en Resultaten", "B31", "Bezetting per woonfunctie")
    read("Projectgegevens en Resultaten", "B32", "P_int;zi [W]")

    # Save tmp-copy + kopieer terug naar bron-pad zodat openpyxl
    # in de toekomst de cached values kan lezen
    wb.Save()
    wb.Close(SaveChanges=False)
finally:
    excel.Quit()
    print("\nExcel closed.")

# Kopieer tmp-versie (met cached values) terug naar bron
shutil.copy(XLSM_TMP, XLSM_SRC)
XLSM_TMP.unlink(missing_ok=True)
print(f"Source xlsm updated with cached values: {XLSM_SRC}")
