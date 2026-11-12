#!/usr/bin/env python3
"""Veille automatique (objectif 1) : ce qui a changé dans les sources suivies.

Lit scripts/veille/sources.json, interroge chaque source (crates.io, GitHub,
flux RSS/Atom), compare avec l'état précédent et écrit un digest Markdown
des nouveautés, groupé par responsable.

    python3 scripts/veille/check_sources.py --state veille-state.json --digest digest.md

- --state  : fichier JSON de l'état précédent (créé s'il n'existe pas, puis mis à jour).
- --digest : digest Markdown des nouveautés.
- --webhook URL (ou variable DISCORD_WEBHOOK_VEILLE) : poste aussi le digest
  sur un salon Discord de l'équipe.

Aucune dépendance hors bibliothèque standard. GITHUB_TOKEN est utilisé s'il
est défini (limite de débit plus haute en CI). Code de sortie 0 même si une
source est injoignable : l'erreur est signalée dans le digest.
"""

import argparse
import json
import os
import sys
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
USER_AGENT = "ferrumos-veille/1.0 (EIP student project)"
TIMEOUT = 20
MAX_ITEMS = 5


def fetch(url, accept=None):
    headers = {"User-Agent": USER_AGENT}
    if accept:
        headers["Accept"] = accept
    token = os.environ.get("GITHUB_TOKEN")
    if token and url.startswith("https://api.github.com/"):
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
        return response.read()


def items_crate(source):
    data = json.loads(fetch(f"https://crates.io/api/v1/crates/{source['name']}"))
    versions = [v for v in data["versions"] if not v["yanked"]][:MAX_ITEMS]
    return [
        {"key": v["num"], "title": f"{source['name']} {v['num']}",
         "date": v["created_at"][:10],
         "url": f"https://crates.io/crates/{source['name']}/{v['num']}"}
        for v in versions
    ]


def items_github_release(source):
    data = json.loads(fetch(
        f"https://api.github.com/repos/{source['repo']}/releases?per_page={MAX_ITEMS}",
        "application/vnd.github+json"))
    return [
        {"key": r["tag_name"], "title": f"{source['repo']} {r['tag_name']}",
         "date": (r.get("published_at") or "")[:10], "url": r["html_url"]}
        for r in data
    ]


def items_github_commits(source):
    data = json.loads(fetch(
        f"https://api.github.com/repos/{source['repo']}/commits?per_page={MAX_ITEMS}",
        "application/vnd.github+json"))
    return [
        {"key": c["sha"], "title": c["commit"]["message"].splitlines()[0][:100],
         "date": c["commit"]["committer"]["date"][:10], "url": c["html_url"]}
        for c in data
    ]


def _text(element, *names):
    for name in names:
        found = element.find(name)
        if found is not None:
            if found.text and found.text.strip():
                return found.text.strip()
            if found.get("href"):
                return found.get("href")
    return ""


def items_rss(source):
    root = ET.fromstring(fetch(source["url"]))
    atom = "{http://www.w3.org/2005/Atom}"
    entries = root.findall("./channel/item") or root.findall(f"{atom}entry")
    items = []
    for entry in entries[:MAX_ITEMS]:
        link = _text(entry, "link", f"{atom}link")
        title = _text(entry, "title", f"{atom}title")
        date = _text(entry, "pubDate", f"{atom}updated", f"{atom}published")
        items.append({"key": _text(entry, "guid", f"{atom}id") or link or title,
                      "title": title, "date": date[:16], "url": link})
    return items


FETCHERS = {
    "crate": items_crate,
    "github_release": items_github_release,
    "github_commits": items_github_commits,
    "rss": items_rss,
}


def check(sources, state):
    """Returns (news, errors, new_state). news: list of (source, items)."""
    news, errors, new_state = [], [], dict(state)
    for source in sources:
        try:
            items = FETCHERS[source["type"]](source)
        except Exception as exc:  # network, format: report and go on
            errors.append((source, f"{type(exc).__name__}: {exc}"))
            continue
        seen = set(state.get(source["id"], []))
        fresh = [item for item in items if item["key"] not in seen]
        # First run for a source: record it without flooding the digest.
        if source["id"] in state and fresh:
            news.append((source, fresh))
        new_state[source["id"]] = sorted(seen | {item["key"] for item in items})
    return news, errors, new_state


def digest(news, errors, sources_count, first_run_ids):
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    lines = [f"# Veille FerrumOS du {today}", ""]
    lines.append(f"{sources_count} sources vérifiées, {sum(len(i) for _, i in news)} nouveautés.")
    lines.append("")
    if not news:
        lines.append("Rien de nouveau depuis le dernier passage.")
    by_owner = {}
    for source, items in news:
        by_owner.setdefault(source["owner"], []).append((source, items))
    for owner in sorted(by_owner):
        lines += ["", f"## Pour {owner}", ""]
        for source, items in by_owner[owner]:
            lines.append(f"**{source['id']}** ({source['area']}) : {source['why']}")
            for item in items:
                lines.append(f"- {item['date']} [{item['title']}]({item['url']})")
            lines.append("")
    if first_run_ids:
        lines += ["", "Sources ajoutées (état initial enregistré) : " + ", ".join(first_run_ids)]
    if errors:
        lines += ["", "## Sources injoignables", ""]
        lines += [f"- {source['id']} : {error}" for source, error in errors]
    lines += ["", "Rappel : chaque nouveauté lue donne une entrée dans "
              "docs/objectif1/03_journal_veille.md, avec une conclusion (même « rien à faire »)."]
    return "\n".join(lines) + "\n"


def post_discord(webhook, text):
    # Discord limits a message to 2000 characters.
    for start in range(0, len(text), 1900):
        body = json.dumps({"content": text[start:start + 1900]}).encode()
        request = urllib.request.Request(
            webhook, data=body,
            headers={"Content-Type": "application/json", "User-Agent": USER_AGENT})
        urllib.request.urlopen(request, timeout=TIMEOUT).read()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--sources", default=str(HERE / "sources.json"))
    parser.add_argument("--state", required=True)
    parser.add_argument("--digest", required=True)
    parser.add_argument("--webhook", default=os.environ.get("DISCORD_WEBHOOK_VEILLE"))
    args = parser.parse_args(argv)

    sources = json.loads(Path(args.sources).read_text(encoding="utf-8"))["sources"]
    state_path = Path(args.state)
    state = json.loads(state_path.read_text(encoding="utf-8")) if state_path.exists() else {}
    first_run_ids = [s["id"] for s in sources if s["id"] not in state]

    news, errors, new_state = check(sources, state)
    text = digest(news, errors, len(sources), first_run_ids)
    Path(args.digest).write_text(text, encoding="utf-8")
    state_path.write_text(json.dumps(new_state, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(text)

    if args.webhook and (news or errors):
        try:
            post_discord(args.webhook, text)
        except Exception as exc:
            print(f"Discord: envoi impossible ({exc})", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
