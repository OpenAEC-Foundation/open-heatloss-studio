"""Genereer/verrijk de KNMI-klimaatbundel voor de vocht-/Glaser-jaarbalans.

Vult `frontend/src/data/climate/knmiClimate.json` met historische jaarrecords
(per station, per kalenderjaar) door KNMI etmaalgegevens (velden TG = etmaal-
gemiddelde temperatuur in 0.1 °C, UG = etmaalgemiddelde relatieve vochtigheid
in %) te aggregeren tot maandgemiddelden.

Twee invoermodi:
  - Online (default): haalt daggegevens op via de KNMI open API
    (https://www.daggegevens.knmi.nl/klimatologie/daggegevens), POST met
    stns=260:240:280:380:235, vars=TG:UG, start/end=YYYYMMDD.
  - Offline: parse lokale `etmgeg_<STN>.txt`-bestanden via --etmgeg-dir.

Het seed-record (260 / 1991-2020 / normal) en het NEN5060-placeholder-record
worden NOOIT overschreven; alleen `kind == "historical"`-records worden
ge-merget (op stationId + selection-jaar).

Gebruik:
    python scripts/generate_climate_bundle.py --start 20210101 --end 20231231
    python scripts/generate_climate_bundle.py --etmgeg-dir ./knmi_raw

Bij netwerk-/parse-falen: de bundel blijft ongewijzigd (seed-only) en het
script rapporteert dat expliciet (geen nep-data).
"""

from __future__ import annotations

import argparse
import calendar
import json
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from collections import defaultdict
from datetime import date
from pathlib import Path

# --- Constanten -------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parent.parent
BUNDLE_PATH = REPO_ROOT / "frontend" / "src" / "data" / "climate" / "knmiClimate.json"

KNMI_API_URL = "https://www.daggegevens.knmi.nl/klimatologie/daggegevens"
STATION_IDS = ["260", "240", "280", "380", "235"]
MONTH_LABELS = [
    "Jan", "Feb", "Mrt", "Apr", "Mei", "Jun",
    "Jul", "Aug", "Sep", "Okt", "Nov", "Dec",
]

# KNMI-sentinel voor ontbrekende waarden + leeg.
_MISSING = {"", "-9999"}


# --- Aggregatie ------------------------------------------------------------


def _days_in_month(year: int, month_idx0: int) -> int:
    """Kalenderdagen in een maand (schrikkeljaar-aware). month_idx0 = 0..11."""
    return calendar.monthrange(year, month_idx0 + 1)[1]


def aggregate_records(rows):
    """Aggregeer (station, datum, TG, UG)-rijen tot historische jaarrecords.

    `rows`: iterable van tuples (station_id:str, year:int, month_idx0:int,
            tg:float|None [0.1 °C], ug:float|None [%]).
    Retourneert een lijst climate-records (kind="historical").
    """
    # acc[(station, year)][month_idx0] = {"tg": [..], "ug": [..]}
    acc = defaultdict(lambda: defaultdict(lambda: {"tg": [], "ug": []}))

    for station_id, year, month_idx0, tg, ug in rows:
        bucket = acc[(station_id, year)][month_idx0]
        if tg is not None:
            bucket["tg"].append(tg)
        if ug is not None:
            bucket["ug"].append(ug)

    records = []
    for (station_id, year), months in sorted(acc.items()):
        month_objs = []
        complete = True
        for m in range(12):
            days = _days_in_month(year, m)
            bucket = months.get(m, {"tg": [], "ug": []})
            tg_vals = bucket["tg"]
            ug_vals = bucket["ug"]
            if not tg_vals or not ug_vals:
                complete = False
                break
            theta_e = round(sum(tg_vals) / len(tg_vals) * 0.1, 1)  # 0.1 °C -> °C
            rh_e = round(sum(ug_vals) / len(ug_vals))
            coverage = round(min(len(tg_vals), len(ug_vals)) / days, 3)
            month_objs.append({
                "month": MONTH_LABELS[m],
                "thetaE": theta_e,
                "rhE": rh_e,
                "days": days,
                "coverage": coverage,
            })
        if not complete:
            print(f"  ! station {station_id} jaar {year}: incomplete maanden, overgeslagen")
            continue
        records.append({
            "stationId": station_id,
            "selection": year,
            "kind": "historical",
            "months": month_objs,
        })
        print(f"  + station {station_id} jaar {year}: 12 maanden geaggregeerd")
    return records


# --- Online: KNMI API -------------------------------------------------------


def fetch_knmi_api(start: str, end: str):
    """Haal daggegevens op via de KNMI open API. Yield (station, year, m0, tg, ug)."""
    payload = urllib.parse.urlencode({
        "stns": ":".join(STATION_IDS),
        "vars": "TG:UG",
        "start": start,
        "end": end,
        "fmt": "json",
    }).encode("utf-8")
    req = urllib.request.Request(KNMI_API_URL, data=payload, method="POST")
    print(f"KNMI API POST {KNMI_API_URL} (stns={','.join(STATION_IDS)}, {start}..{end})")
    with urllib.request.urlopen(req, timeout=60) as resp:
        raw = resp.read().decode("utf-8")

    data = json.loads(raw)
    # API levert een lijst dicts met o.a. station_code/STN, date, TG, UG.
    for entry in data:
        station = str(entry.get("station_code") or entry.get("STN") or "").strip()
        date_str = str(entry.get("date") or entry.get("YYYYMMDD") or "")
        # date kan "2021-01-01T00:00:00.000Z" of "20210101" zijn.
        digits = re.sub(r"\D", "", date_str)[:8]
        if station not in STATION_IDS or len(digits) != 8:
            continue
        year = int(digits[:4])
        month_idx0 = int(digits[4:6]) - 1
        tg = _to_float(entry.get("TG"))
        ug = _to_float(entry.get("UG"))
        yield station, year, month_idx0, tg, ug


def _to_float(val):
    if val is None:
        return None
    s = str(val).strip()
    if s in _MISSING:
        return None
    try:
        return float(s)
    except ValueError:
        return None


# --- Offline: etmgeg_<STN>.txt ---------------------------------------------


def parse_etmgeg_dir(directory: Path):
    """Parse alle etmgeg_<STN>.txt in `directory`. Yield (station, year, m0, tg, ug).

    KNMI etmgeg-formaat: comma-separated, headerregels beginnen met '#'.
    Kolommen (relevant): STN, YYYYMMDD, ..., TG, ..., UG. We lezen de
    laatste '# STN,YYYYMMDD,...'-headerregel om kolomposities te bepalen.
    """
    files = sorted(directory.glob("etmgeg_*.txt"))
    if not files:
        raise FileNotFoundError(f"Geen etmgeg_<STN>.txt in {directory}")
    for path in files:
        print(f"  parse {path.name}")
        yield from _parse_etmgeg_file(path)


def _parse_etmgeg_file(path: Path):
    header_cols = None
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if line.startswith("#"):
                body = line.lstrip("#").strip()
                if body.upper().startswith("STN"):
                    header_cols = [c.strip().upper() for c in body.split(",")]
                continue
            if not line.strip() or header_cols is None:
                continue
            parts = [p.strip() for p in line.split(",")]
            if len(parts) < len(header_cols):
                continue
            row = dict(zip(header_cols, parts))
            station = row.get("STN", "")
            date_str = re.sub(r"\D", "", row.get("YYYYMMDD", ""))[:8]
            if station not in STATION_IDS or len(date_str) != 8:
                continue
            year = int(date_str[:4])
            month_idx0 = int(date_str[4:6]) - 1
            yield station, year, month_idx0, _to_float(row.get("TG")), _to_float(row.get("UG"))


# --- Merge ------------------------------------------------------------------


def merge_records(bundle: dict, new_records: list) -> int:
    """Merge historische records in de bundel. Seed/placeholder blijven intact.

    Retourneert het aantal toegevoegde/vervangen records.
    """
    existing = bundle.setdefault("records", [])

    def is_protected(rec):
        return rec.get("kind") in ("normal", "reference")

    # Index bestaande historische records op (stationId, selection).
    kept = [r for r in existing if is_protected(r) or r.get("kind") == "historical"]
    by_key = {}
    out = []
    for r in kept:
        if r.get("kind") == "historical":
            by_key[(r["stationId"], r["selection"])] = r
        else:
            out.append(r)  # protected first

    count = 0
    for rec in new_records:
        by_key[(rec["stationId"], rec["selection"])] = rec
        count += 1

    # Sorteer historische records: per station, jaar oplopend.
    hist = sorted(by_key.values(), key=lambda r: (r["stationId"], r["selection"]))
    bundle["records"] = out + hist
    return count


# --- Main -------------------------------------------------------------------


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--start", default="20210101", help="Begindatum YYYYMMDD (online)")
    ap.add_argument("--end", default="20231231", help="Einddatum YYYYMMDD (online)")
    ap.add_argument("--etmgeg-dir", type=Path, default=None,
                    help="Map met lokale etmgeg_<STN>.txt (offline modus)")
    ap.add_argument("--bundle", type=Path, default=BUNDLE_PATH,
                    help="Pad naar knmiClimate.json")
    args = ap.parse_args()

    bundle_path: Path = args.bundle
    if not bundle_path.exists():
        print(f"FOUT: bundel niet gevonden: {bundle_path}", file=sys.stderr)
        return 2
    bundle = json.loads(bundle_path.read_text(encoding="utf-8"))

    try:
        if args.etmgeg_dir is not None:
            rows = list(parse_etmgeg_dir(args.etmgeg_dir))
        else:
            rows = list(fetch_knmi_api(args.start, args.end))
    except (urllib.error.URLError, OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"\nKNMI-data ophalen/parsen MISLUKT: {exc!r}", file=sys.stderr)
        print("Bundel blijft ongewijzigd (seed-only). Geen nep-data geschreven.",
              file=sys.stderr)
        return 1

    if not rows:
        print("\nGeen bruikbare rijen ontvangen. Bundel ongewijzigd (seed-only).",
              file=sys.stderr)
        return 1

    new_records = aggregate_records(rows)
    if not new_records:
        print("\nGeen complete jaarrecords. Bundel ongewijzigd (seed-only).",
              file=sys.stderr)
        return 1

    added = merge_records(bundle, new_records)

    meta = bundle.setdefault("_meta", {})
    meta["knmi_download_date"] = date.today().isoformat()

    bundle_path.write_text(
        json.dumps(bundle, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"\nKlaar: {added} historisch(e) record(s) ge-merget in {bundle_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
