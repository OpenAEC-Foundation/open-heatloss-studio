#!/usr/bin/env python
"""Check if any text content in a PDF crosses into the footer zone.

The brand callback in reports/brand.rs draws its running footer at
y = page_h - 12mm (line) and y = page_h - 10mm (text). Anything content
flowable that lands below ~272mm on A4 is suspicious.

Usage:
    python tools/check_pdf_overflow.py path/to/report.pdf [--threshold-mm 272]
"""
import argparse
import sys
import io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

import fitz  # type: ignore


# Known running chrome text. Brand callback emits these on every page;
# they're not content overflow.
CHROME_PATTERNS = (
    "Open Heatloss Studio",
    "Warmteverliesberekening conform ISSO",
)


def is_chrome(text: str, project_name: str) -> bool:
    t = text.strip()
    if not t:
        return True
    # Top-left header: the project name itself
    if t == project_name:
        return True
    # Top-right header
    for p in CHROME_PATTERNS:
        if p in t:
            return True
    # "n / m" page counter (e.g. "5 / 22"); short token with slash
    if "/" in t and len(t) < 12 and any(c.isdigit() for c in t):
        return True
    return False


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("pdf")
    ap.add_argument(
        "--threshold-pt",
        type=float,
        default=None,
        help="y-coordinate (top-down, pt) below which content counts as overflow. "
        "Defaults to page_height − 22pt (about 7.7mm above bottom edge), matching "
        "the footer-line + 4pt margin.",
    )
    ap.add_argument("--project-name", default="Memeleiland Kavel 4")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    doc = fitz.open(args.pdf)
    print(f"Inspecting {args.pdf} — {len(doc)} pages")

    total_overflow = 0
    pages_with_overflow = []
    for i in range(len(doc)):
        page = doc[i]
        ph = page.rect.height
        threshold = args.threshold_pt if args.threshold_pt is not None else ph - 22.0
        # Footer text typically sits at y ≈ ph - 10mm = ph - 28.3pt; we want
        # to flag anything whose BOTTOM extends past threshold AND which is NOT
        # one of the known chrome lines.

        offenders = []
        for block in page.get_text("dict")["blocks"]:
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    bbox = span["bbox"]
                    txt = span["text"]
                    if is_chrome(txt, args.project_name):
                        continue
                    # We want content whose bottom (bbox[3], y-down) > threshold
                    # but whose top is reasonably within the page (excludes
                    # weird negative coords).
                    if bbox[3] > threshold and bbox[1] < ph - 5:
                        offenders.append((bbox, txt.strip()))

        if offenders:
            total_overflow += len(offenders)
            pages_with_overflow.append(i + 1)
            if args.verbose:
                print(f"  page {i+1:3d}: {len(offenders)} overflowing span(s) (threshold y > {threshold:.0f}pt)")
                for bb, t in offenders[:5]:
                    print(f"    y={bb[1]:6.1f}-{bb[3]:6.1f}  text={t!r}")

    if not pages_with_overflow:
        print(f"\n✓ No content overflow detected (threshold = ph − 22pt).")
        return 0
    print(
        f"\n✗ Overflow on {len(pages_with_overflow)} page(s) "
        f"(pages: {pages_with_overflow}; {total_overflow} total span(s)).",
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
