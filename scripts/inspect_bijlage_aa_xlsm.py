"""Inspecteer welke cellen invoer/output zijn in de bijlage-aa-sample-case1 xlsm."""

import io
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

import openpyxl
from pathlib import Path

XLSM = Path(__file__).parent.parent / "tests" / "references" / "bijlage-aa-sample-case1-slaapkamer-zuid.xlsm"

wb = openpyxl.load_workbook(XLSM, keep_vba=True, data_only=False)

for sheet_name in ["Projectgegevens en Resultaten", "Ruimte 1"]:
    ws = wb[sheet_name]
    print(f"\n=== Sheet: {sheet_name} (dim={ws.dimensions}) ===")
    for row in ws.iter_rows(min_row=1, max_row=70, max_col=3, values_only=False):
        for cell in row:
            if cell.value is None:
                continue
            val = str(cell.value)
            if len(val) > 100:
                val = val[:97] + "..."
            print(f"  {cell.coordinate:6s} | {val}")
