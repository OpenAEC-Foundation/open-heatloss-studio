"""Extract VBA o_F_Iso function source from xlsm."""

import io
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

from pathlib import Path
from oletools.olevba import VBA_Parser

XLSM = Path(__file__).parent.parent / "tests" / "references" / "bijlage-aa-sample-case1-slaapkamer-zuid.xlsm"
v = VBA_Parser(str(XLSM))

for filename, stream_path, vba_filename, vba_code in v.extract_macros():
    if "F_Iso" in vba_code or "f_iso" in vba_code.lower():
        print(f"=== {vba_filename} ===")
        # Print alleen relevante sectie rond F_Iso
        lines = vba_code.split("\n")
        for i, line in enumerate(lines):
            if "f_iso" in line.lower():
                start = max(0, i - 2)
                end = min(len(lines), i + 35)
                print(f"\n--- lines {start+1}-{end} ---")
                for j in range(start, end):
                    print(f"  {j+1:4d} | {lines[j]}")
                print()
