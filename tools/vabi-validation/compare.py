#!/usr/bin/env python3
"""Compare our ISSO 51 calculation against a Vabi reference, per room.

Two inputs:

1. **Our calculation** — produced by running our ISSO 51 engine on a project
   JSON. The engine is invoked via the Cargo example
   ``cargo run --example calc_from_file -- <project.json>`` (the canonical
   debug entry point; it prints the full result JSON to stdout). This script
   does NOT touch the Rust calculation kernel — it only shells out to it.

   Alternatively, pass ``--our-result <result.json>`` with a pre-computed result
   JSON (same shape the example prints) to skip the Cargo build.

2. **Vabi reference** — a ``reference_*.json`` filled by hand from a Vabi
   warmteverlies PDF report. See ``reference_template.json`` for the schema and
   ``reference_portiekwoning_example.json`` for a worked example.

For each matched room it prints Phi_transmissie, Phi_ventilatie+infiltratie,
Phi_totaal and H_T with absolute and percentage differences, and a PASS/FAIL on
a tolerance (default 5%).

Reference JSON format (per room, all powers in W, H_T in W/K)::

    {
      "project": "ISSO 51 Portiekwoning",
      "source": "Vabi Elements WV-rapport PDF",
      "tolerance_pct": 5.0,
      "rooms": {
        "<room name or id>": {
          "phi_transmission": 783.7,
          "phi_ventilation": 667.7,   # ventilation + infiltration combined
          "phi_total": 1391.8,
          "h_t": 30.1
        }
      }
    }

Room matching is by exact id first, then by case-insensitive name (the Vabi room
names carry a numeric prefix like ``01:Woonkamer`` which is stripped for the
match).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_TOLERANCE_PCT = 5.0
NAME_PREFIX_RE = re.compile(r"^\s*\d+\s*[:.\-]\s*")


@dataclass
class RoomMetrics:
    """The four compared quantities for one room (W, W, W, W/K)."""

    phi_transmission: float
    phi_ventilation: float  # ventilation + infiltration combined
    phi_total: float
    h_t: float


# --- Running / loading our calculation -------------------------------------


def run_our_calculation(project_json: Path) -> dict:
    """Invoke the Cargo example and return the parsed result JSON."""
    # Resolve to an absolute path: the subprocess runs from REPO_ROOT, so a
    # path relative to the caller's cwd would not be found.
    cmd = [
        "cargo", "run", "--quiet", "--example", "calc_from_file",
        "--", str(project_json.resolve()),
    ]
    proc = subprocess.run(
        cmd, cwd=str(REPO_ROOT), capture_output=True, text=True, check=False
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"calc_from_file failed (exit {proc.returncode}):\n{proc.stderr}"
        )
    return _extract_result_json(proc.stdout)


def _extract_result_json(stdout: str) -> dict:
    """Pull the result JSON object out of the example's stdout.

    The example prints '=== Full result JSON ===' then the pretty JSON, then a
    '=== Summary ===' text block. We slice the first complete JSON object.
    """
    marker = "=== Full result JSON ==="
    start = stdout.find(marker)
    if start != -1:
        start = stdout.find("{", start)
    else:
        start = stdout.find("{")
    if start == -1:
        raise ValueError("No JSON object found in calc_from_file output")
    depth = 0
    for i in range(start, len(stdout)):
        ch = stdout[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return json.loads(stdout[start : i + 1])
    raise ValueError("Unbalanced JSON in calc_from_file output")


def load_our_result(result_json: Path) -> dict:
    return json.loads(result_json.read_text(encoding="utf-8"))


def our_room_metrics(result: dict) -> dict[str, RoomMetrics]:
    """Index our per-room results by both id and normalised name."""
    out: dict[str, RoomMetrics] = {}
    for room in result.get("rooms", []):
        transmission = room.get("transmission", {}) or {}
        ventilation = room.get("ventilation", {}) or {}
        infiltration = room.get("infiltration", {}) or {}

        phi_t = float(transmission.get("phi_t", 0.0) or 0.0)
        phi_v = float(ventilation.get("phi_v", 0.0) or 0.0)
        phi_inf = float(infiltration.get("phi_i", 0.0) or 0.0)
        phi_total = float(room.get("total_heat_loss", 0.0) or 0.0)

        # H_T = sum of all transmission conductance components (W/K).
        h_t = sum(
            float(transmission.get(key, 0.0) or 0.0)
            for key in (
                "h_t_exterior", "h_t_unheated", "h_t_adjacent_rooms",
                "h_t_adjacent_buildings", "h_t_ground", "h_t_water",
            )
        )

        metrics = RoomMetrics(
            phi_transmission=phi_t,
            phi_ventilation=phi_v + phi_inf,
            phi_total=phi_total,
            h_t=h_t,
        )
        room_id = str(room.get("room_id", "")).strip()
        room_name = str(room.get("room_name", "")).strip()
        if room_id:
            out[_norm_key(room_id)] = metrics
        if room_name:
            out[_norm_key(room_name)] = metrics
    return out


# --- Reference loading -----------------------------------------------------


def load_reference(reference_json: Path) -> tuple[dict[str, RoomMetrics], float, dict]:
    raw = json.loads(reference_json.read_text(encoding="utf-8"))
    tolerance = float(raw.get("tolerance_pct", DEFAULT_TOLERANCE_PCT))
    rooms: dict[str, RoomMetrics] = {}
    for name, vals in raw.get("rooms", {}).items():
        rooms[_norm_key(name)] = RoomMetrics(
            phi_transmission=float(vals.get("phi_transmission", 0.0)),
            phi_ventilation=float(vals.get("phi_ventilation", 0.0)),
            phi_total=float(vals.get("phi_total", 0.0)),
            h_t=float(vals.get("h_t", 0.0)),
        )
    return rooms, tolerance, raw


def _norm_key(name: str) -> str:
    """Normalise a room key: strip numeric prefix, lowercase, collapse space."""
    stripped = NAME_PREFIX_RE.sub("", name)
    return stripped.strip().lower()


# --- Comparison table ------------------------------------------------------


def _diff(ours: float, ref: float) -> tuple[float, Optional[float]]:
    abs_diff = ours - ref
    pct = (abs_diff / ref * 100.0) if ref != 0.0 else None
    return abs_diff, pct


def _fmt_pct(pct: Optional[float]) -> str:
    return "n/a" if pct is None else f"{pct:+.1f}%"


def _status(pct: Optional[float], tolerance: float) -> str:
    if pct is None:
        return "n/a"
    return "PASS" if abs(pct) <= tolerance else "FAIL"


def compare(
    ours: dict[str, RoomMetrics],
    reference: dict[str, RoomMetrics],
    tolerance: float,
) -> int:
    """Print the comparison table. Returns the number of FAIL rows."""
    quantities = [
        ("Phi_T", "phi_transmission", "W"),
        ("Phi_V+inf", "phi_ventilation", "W"),
        ("Phi_tot", "phi_total", "W"),
        ("H_T", "h_t", "W/K"),
    ]
    header = (
        f"{'Room':<22}{'Quantity':<11}{'Ours':>12}{'Vabi-ref':>12}"
        f"{'AbsDiff':>12}{'%Diff':>9}  {'Status'}"
    )
    print(header)
    print("-" * len(header))

    fails = 0
    for ref_key in sorted(reference.keys()):
        ref_metrics = reference[ref_key]
        our_metrics = ours.get(ref_key)
        room_label = ref_key
        if our_metrics is None:
            print(f"{room_label:<22}{'(no match in our result)':<44}  SKIP")
            continue
        first = True
        for label, attr, unit in quantities:
            ours_v = getattr(our_metrics, attr)
            ref_v = getattr(ref_metrics, attr)
            abs_diff, pct = _diff(ours_v, ref_v)
            status = _status(pct, tolerance)
            if status == "FAIL":
                fails += 1
            name_col = room_label if first else ""
            first = False
            print(
                f"{name_col:<22}{label + ' [' + unit + ']':<11}"
                f"{ours_v:>12.1f}{ref_v:>12.1f}{abs_diff:>12.1f}"
                f"{_fmt_pct(pct):>9}  {status}"
            )
        print()

    # Rooms we computed but the reference does not cover.
    extra = sorted(set(ours.keys()) - set(reference.keys()))
    # de-dup id/name aliases pointing at the same metrics is hard here; just note count
    if extra:
        print(f"Note: {len(extra)} of our room keys had no reference entry "
              f"(id/name aliases included).")

    print(f"\nTolerance: {tolerance:.1f}%   FAIL rows: {fails}")
    return fails


# --- CLI -------------------------------------------------------------------


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare our ISSO 51 result against a Vabi reference."
    )
    parser.add_argument(
        "reference", type=Path, help="Path to a reference_*.json file"
    )
    src = parser.add_mutually_exclusive_group(required=True)
    src.add_argument(
        "--project", type=Path,
        help="Project JSON to run through our engine via cargo calc_from_file",
    )
    src.add_argument(
        "--our-result", type=Path,
        help="Pre-computed result JSON (skip cargo build)",
    )
    parser.add_argument(
        "--tolerance", type=float, default=None,
        help=f"Override tolerance %% (default: from reference or {DEFAULT_TOLERANCE_PCT})",
    )
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    # Windows consoles default to cp1252; force UTF-8 so em-dashes etc. survive.
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is not None:
            reconfigure(encoding="utf-8")

    args = parse_args(argv)

    reference, tol_from_ref, raw = load_reference(args.reference)
    tolerance = args.tolerance if args.tolerance is not None else tol_from_ref

    if args.our_result is not None:
        result = load_our_result(args.our_result)
    else:
        print(f"Running ISSO 51 engine on {args.project} ...", file=sys.stderr)
        result = run_our_calculation(args.project)

    ours = our_room_metrics(result)

    print(f"Reference: {raw.get('project', args.reference.name)} "
          f"(source: {raw.get('source', 'unknown')})\n")
    fails = compare(ours, reference, tolerance)
    return 1 if fails > 0 else 0


if __name__ == "__main__":
    raise SystemExit(main())
