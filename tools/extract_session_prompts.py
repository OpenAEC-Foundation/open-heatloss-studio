"""Extract alle user-prompts uit de Claude Code session-transcript JSONL.

Minimal filtering: alleen expliciete runtime-injecties (system-reminders,
tool-results, skill-activations, slash-command meta) worden weggehaald.
Adjacent-duplicates wel gededuplicaten, herhaalde prompts met tijd-gat
blijven staan.

Usage:
    python tools/extract_session_prompts.py
"""
import json
import datetime as dt
from pathlib import Path

SRC = Path(
    r"C:\Users\rickd\.claude\projects"
    r"\C--Users-rickd-Documents-GitHub-open-heatloss-studio--claude-worktrees-laughing-kirch-752da4"
    r"\59533232-5ad1-4010-8f79-1ab6ad54b654.jsonl"
)
OUT = Path("docs/sessie-prompts-week-mei-2026.md")


def is_noise(text: str) -> bool:
    """Filter alleen expliciete runtime-injected user-messages."""
    sysreminder_open = "<" + "system-reminder" + ">"
    if text.startswith(sysreminder_open):
        return True
    if text.startswith("<local-command-stdout>") or text.startswith("<command-name>"):
        return True
    if text.startswith("<command-message>"):
        return True
    if text.startswith("Caveat:"):
        return True
    if text.startswith("This session is being continued"):
        return True
    if text.startswith("Launching skill:"):
        return True
    if text.startswith("Base directory for this skill:"):
        return True
    return False


def main() -> None:
    users = []
    recent: list[str] = []
    with open(SRC, "r", encoding="utf-8") as f:
        for line in f:
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            if obj.get("type") != "user":
                continue
            if obj.get("isSidechain"):
                continue
            msg = obj.get("message", {})
            content = msg.get("content")
            if isinstance(content, str):
                text = content
            elif isinstance(content, list):
                parts = [
                    blk.get("text", "")
                    for blk in content
                    if isinstance(blk, dict) and blk.get("type") == "text"
                ]
                text = "\n".join(p for p in parts if p)
            else:
                continue
            text = (text or "").strip()
            if not text or is_noise(text):
                continue
            # adjacent dedup only
            if recent and recent[-1] == text:
                continue
            recent.append(text)
            if len(recent) > 5:
                recent.pop(0)
            users.append({"ts": obj.get("timestamp", ""), "text": text})

    users.sort(key=lambda x: x["ts"])

    def to_local(ts_str: str) -> dt.datetime:
        base = dt.datetime.strptime(ts_str.replace("Z", ""), "%Y-%m-%dT%H:%M:%S.%f")
        return base + dt.timedelta(hours=2)

    by_day: dict[str, list] = {}
    for u in users:
        local = to_local(u["ts"])
        key = local.strftime("%Y-%m-%d")
        by_day.setdefault(key, []).append(
            {"time": local.strftime("%H:%M"), "text": u["text"]}
        )

    day_labels_nl = {
        0: "maandag", 1: "dinsdag", 2: "woensdag", 3: "donderdag",
        4: "vrijdag", 5: "zaterdag", 6: "zondag",
    }
    month_nl = {
        1: "januari", 2: "februari", 3: "maart", 4: "april", 5: "mei",
        6: "juni", 7: "juli", 8: "augustus", 9: "september", 10: "oktober",
        11: "november", 12: "december",
    }

    lines: list[str] = []
    lines.append("# Sessie-prompts week 8-15 mei 2026")
    lines.append("")
    lines.append(
        "Alle user-prompts in de doorlopende Claude Code sessie op de "
        "worktree-branch `claude/laughing-kirch-752da4`."
    )
    lines.append("")
    lines.append(
        f"**Periode:** {users[0]['ts'][:10]} t/m {users[-1]['ts'][:10]} — "
        f"**{len(users)} prompts** gegroepeerd per dag (Amsterdam-tijd)."
    )
    lines.append("")
    lines.append(
        "Geëxtraheerd uit de session-transcript JSONL. Minimal filtering: "
        "alleen expliciete system-reminders, tool-results, slash-command "
        "metadata en skill-activation dumps weggehaald. Adjacent-duplicates "
        "geneerd; herhaalde prompts met tijd-gat blijven staan."
    )
    lines.append("")
    lines.append("---")
    lines.append("")

    for day in sorted(by_day.keys()):
        dt_obj = dt.datetime.strptime(day, "%Y-%m-%d")
        weekday = day_labels_nl[dt_obj.weekday()]
        nice = f"{weekday} {dt_obj.day} {month_nl[dt_obj.month]}"
        prompts = by_day[day]
        lines.append(f"## {nice} ({day}) — {len(prompts)} prompts")
        lines.append("")
        for i, p in enumerate(prompts, 1):
            text = p["text"]
            if len(text) > 1500:
                text = text[:1500].rstrip() + f"\n\n*[afgekapt, was {len(p['text'])} chars]*"
            if "\n" in text:
                quoted = "\n".join("> " + ln for ln in text.split("\n"))
                lines.append(f"**{i}. [{p['time']}]**")
                lines.append("")
                lines.append(quoted)
            else:
                lines.append(f"**{i}. [{p['time']}]** {text}")
            lines.append("")
        lines.append("---")
        lines.append("")

    themes = {
        "2026-05-08": "Installer-setup + PDF rapport eerste opzet",
        "2026-05-09": "PDF overflow fixes, rapport-features uitbreiden",
        "2026-05-10": "Recent files, .ifcenergy associatie",
        "2026-05-11": "Marges, voorpaginafbeelding, dark-mode dropdowns",
        "2026-05-12": "IFCX validatie, Save vs Save As scheiding",
        "2026-05-13": "Master-pull, TO-juli, Vabi-importer, crates-warehouse",
        "2026-05-14": "Stille dag",
        "2026-05-15": "Rapport-overhaul, tabbed views, OpenAEC branding, PR",
    }
    lines.append("## Samenvatting per dag")
    lines.append("")
    lines.append("| Datum | Prompts | Hoofdthema |")
    lines.append("|---|---|---|")
    for day in sorted(by_day.keys()):
        cnt = len(by_day[day])
        theme = themes.get(day, "(diverse)")
        lines.append(f"| {day} | {cnt} | {theme} |")
    lines.append("")
    lines.append("## Branch-resultaat")
    lines.append("")
    lines.append("- **128 commits** sinds master")
    lines.append("- **243 files changed**, 33k insertions / 5k deletions")
    lines.append("- **PR #13**: https://github.com/OpenAEC-Foundation/open-heatloss-studio/pull/13")

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Prompts: {len(users)}")
    print(f"Wrote {OUT} ({OUT.stat().st_size} bytes)")
    for day in sorted(by_day.keys()):
        print(f"  {day}: {len(by_day[day])} prompts")


if __name__ == "__main__":
    main()
