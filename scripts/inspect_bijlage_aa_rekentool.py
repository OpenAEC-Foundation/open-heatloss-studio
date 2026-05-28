"""Verken cell-structuur van RVO Rekentool Bijlage AA xlsm.

Dumpt voor 'Ruimte 1' en 'Projectgegevens en Resultaten' alle cellen
met label/waarde/formule. Output naar stdout (UTF-8 forced).
"""
import sys
import openpyxl
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

XLSM = Path(r"C:/Github/open-heatloss-studio/tests/references/rekentool-bijlage-aa-nta8800-2025.04.xlsm")

wb = openpyxl.load_workbook(XLSM, keep_vba=True, data_only=False)


def dump_sheet(sheet_name: str, max_row: int = 100):
    ws = wb[sheet_name]
    print(f"\n=== {sheet_name} (dim={ws.dimensions}) ===")
    for row in ws.iter_rows(min_row=1, max_row=max_row):
        for c in row:
            if c.value is None:
                continue
            val = str(c.value)
            # Indicator: F=formule, V=waarde
            kind = "F" if val.startswith("=") else "V"
            # Truncate long values
            disp = val if len(val) < 100 else val[:97] + "..."
            print(f"  {c.coordinate:5s} [{kind}] {disp}")


dump_sheet("Ruimte 1", max_row=90)
print()
dump_sheet("Projectgegevens en Resultaten", max_row=80)
