#!/usr/bin/env python3
"""Collect and compare FerrumOS kernel test results (QEMU).

Two sub-commands, standard library only:

  collect  Parse the output of `cargo test` (serial output of each QEMU
           image + cargo's own lines) into a results JSON file.
  compare  Check a results file against the baseline rules, optionally show
           the difference with a previous run, and write a Markdown report.
           Exit code 1 when a regression or a failed test is found.
  history  Append the measurements of a run to a CSV file (one row per
           RECORD), to follow the values over time.

Only deterministic values (iterations, errors, memory used) can be ruled in
the baseline. Cycle counts are emulated by QEMU and vary by up to 20 % between
identical runs, so they are reported but never used to fail a run.
"""

import argparse
import json
import platform
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

FORMAT_VERSION = 1

# cargo: "     Running unittests src/lib.rs (target/...)" or
#        "     Running tests/experiment_2.rs (target/...)"
SUITE_RE = re.compile(r"^\s*Running (?:unittests )?(\S+\.rs)\s+\(")
# cargo, when a target fails: "error: test failed, to rerun pass `--test experiment_2`"
# and in the --no-fail-fast summary: "    `--test experiment_2`" / "    `--lib`"
RERUN_RE = re.compile(r"`--(lib|test (\S+?))`")
# kernel test runner (src/testing.rs): "ferrumos::tests::name...\t"
TEST_START_RE = re.compile(r"^(\S+)\.\.\.\t")
# kernel measurement line (testing::Record::report)
RECORD_RE = re.compile(r"\bRECORD\s+name=(\S+)((?:\s+\w+=-?\d+)*)")
FIELD_RE = re.compile(r"(\w+)=(-?\d+)")
# kernel test runner: "Running 11 tests"
TEST_COUNT_RE = re.compile(r"^Running (\d+) tests\s*$")


def suite_id(path):
    """`src/lib.rs` -> `lib`, `tests/experiment_2.rs` -> `experiment_2`."""
    if path.endswith("src/lib.rs"):
        return "lib"
    return Path(path).stem


def parse_log(text):
    """Parse cargo test output. Returns (suites, records) in run order."""
    suites = []
    records = []
    current = None
    test = None
    failed_targets = set()

    for raw in text.splitlines():
        line = raw.rstrip("\r")

        m = SUITE_RE.match(line)
        if m:
            current = {
                "id": suite_id(m.group(1)),
                "source": m.group(1),
                "status": "passed",
                "declared_tests": None,
                "tests": [],
                "panic": None,
            }
            suites.append(current)
            test = None
            continue

        for m in RERUN_RE.finditer(line):
            failed_targets.add("lib" if m.group(1) == "lib" else m.group(2))

        if current is None:
            continue

        m = TEST_COUNT_RE.match(line)
        if m:
            current["declared_tests"] = int(m.group(1))

        m = TEST_START_RE.match(line)
        if m:
            test = {"name": m.group(1), "status": "running"}
            current["tests"].append(test)

        m = RECORD_RE.search(line)
        if m:
            fields = {k: int(v) for k, v in FIELD_RE.findall(m.group(2))}
            records.append({"name": m.group(1), "suite": current["id"], **fields})

        if "[failed]" in line:
            current["status"] = "failed"
            if test is not None and test["status"] == "running":
                test["status"] = "failed"
            test = None
            continue
        if current["status"] == "failed" and current["panic"] is None and line.startswith("panicked"):
            current["panic"] = line.strip()
        if "[ok]" in line and test is not None:
            test["status"] = "passed"
            test = None

    for suite in suites:
        if suite["id"] in failed_targets:
            suite["status"] = "failed"
        # a test that started but never printed [ok]/[failed]: the image hung,
        # timed out or crashed without reaching the panic handler.
        for t in suite["tests"]:
            if t["status"] == "running":
                t["status"] = "unfinished"
                suite["status"] = "failed"
    return suites, records


def command_output(cmd):
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        return out.stdout.strip().splitlines()[0] if out.stdout.strip() else None
    except (OSError, subprocess.SubprocessError):
        return None


def cmd_collect(args):
    text = Path(args.log).read_text(encoding="utf-8", errors="replace")
    suites, records = parse_log(text)

    if args.exit_code != 0 and not any(s["status"] == "failed" for s in suites):
        # cargo failed but no test target reported a failure: build error,
        # missing toolchain, runner crash...
        status = "error"
    elif args.exit_code != 0 or any(s["status"] == "failed" for s in suites):
        status = "failed"
    else:
        status = "passed"

    results = {
        "format_version": FORMAT_VERSION,
        "status": status,
        "cargo_exit_code": args.exit_code,
        "run": {
            "date": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "commit": args.commit or command_output(["git", "rev-parse", "HEAD"]),
            "branch": args.branch or command_output(["git", "rev-parse", "--abbrev-ref", "HEAD"]),
            "source": args.source,
        },
        "environment": {
            "rustc": command_output(["rustc", "--version"]),
            "bootimage": command_output(["bootimage", "--version"]),
            "qemu": command_output(["qemu-system-x86_64", "--version"]),
            "host": platform.platform(),
        },
        "suites": suites,
        "records": records,
    }
    Path(args.output).write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
    print(f"{len(suites)} test images, {len(records)} records, status: {status} -> {args.output}")
    return 0


def check_rule(value, rule):
    """Returns None when the value respects the rule, else a description."""
    if value is None:
        return "missing"
    if "min" in rule and value < rule["min"]:
        return f"< {rule['min']}"
    if "max" in rule and value > rule["max"]:
        return f"> {rule['max']}"
    if "equals" in rule and value != rule["equals"]:
        return f"!= {rule['equals']}"
    return None


def rule_text(rule):
    parts = []
    if "min" in rule:
        parts.append(f">= {rule['min']}")
    if "max" in rule:
        parts.append(f"<= {rule['max']}")
    if "equals" in rule:
        parts.append(f"= {rule['equals']}")
    return " and ".join(parts)


def last_by_name(records):
    out = {}
    for r in records:
        out[r["name"]] = r
    return out


def fmt_delta(current, previous):
    if current is None or previous is None:
        return ""
    delta = current - previous
    if delta == 0:
        return "="
    pct = f" ({delta / previous:+.0%})" if previous else ""
    return f"{delta:+d}{pct}"


def compare(results, baseline, previous=None):
    """Returns (problems, markdown report lines)."""
    problems = []
    lines = []
    current = last_by_name(results.get("records", []))
    prev = last_by_name(previous.get("records", [])) if previous else {}

    run = results.get("run", {})
    commit = (run.get("commit") or "?")[:7]
    lines.append(f"## Tests noyau QEMU — `{commit}`")
    lines.append("")

    # 1. test images
    if results.get("status") == "error":
        problems.append("cargo test failed before running the tests (build or runner error)")
    lines.append("| Image de test | Statut | Tests |")
    lines.append("|---|---|---|")
    for s in results.get("suites", []):
        passed = sum(1 for t in s["tests"] if t["status"] == "passed")
        total = len(s["tests"])
        tests = f"{passed}/{total}" if total else "—"
        mark = "✅" if s["status"] == "passed" else "❌"
        lines.append(f"| `{s['id']}` | {mark} {s['status']} | {tests} |")
        if s["status"] != "passed":
            bad = [t["name"] for t in s["tests"] if t["status"] != "passed"]
            detail = f" ({', '.join(bad)})" if bad else ""
            problems.append(f"test image `{s['id']}` {s['status']}{detail}")
    for sid in baseline.get("expected_suites", []):
        if sid not in {s["id"] for s in results.get("suites", [])}:
            problems.append(f"test image `{sid}` did not run")
            lines.append(f"| `{sid}` | ❌ did not run | — |")
    lines.append("")

    # 2. baseline rules
    lines.append("| Mesure | Règle (baseline) | Précédent | Actuel | Écart | Statut |")
    lines.append("|---|---|---|---|---|---|")
    for name, rules in baseline.get("records", {}).items():
        rec = current.get(name)
        if rec is None:
            problems.append(f"{name}: no RECORD line in the output")
        for field, rule in rules.items():
            value = rec.get(field) if rec else None
            pvalue = prev.get(name, {}).get(field)
            issue = check_rule(value, rule)
            status = "✅" if issue is None else f"❌ {issue}"
            if issue is not None and rec is not None:
                problems.append(f"{name}.{field} = {value} ({issue}, rule {rule_text(rule)})")
            shown = "—" if value is None else value
            shown_prev = "" if pvalue is None else pvalue
            lines.append(
                f"| `{name}.{field}` | {rule_text(rule)} | {shown_prev} | {shown} "
                f"| {fmt_delta(value, pvalue)} | {status} |"
            )
    lines.append("")

    # 3. informational values (never fail the run)
    info = [(n, r) for n, r in current.items() if "cycles" in r]
    if info:
        lines.append("<details><summary>Cycles (indicatif, ±20 % sous QEMU, jamais bloquant)</summary>")
        lines.append("")
        lines.append("| Mesure | Précédent | Actuel | Écart |")
        lines.append("|---|---|---|---|")
        for name, rec in info:
            pvalue = prev.get(name, {}).get("cycles")
            lines.append(
                f"| `{name}.cycles` | {'' if pvalue is None else pvalue} | {rec['cycles']} "
                f"| {fmt_delta(rec['cycles'], pvalue)} |"
            )
        lines.append("")
        lines.append("</details>")
        lines.append("")

    unknown = sorted(set(current) - set(baseline.get("records", {})))
    if unknown:
        lines.append(f"Mesures sans règle dans la baseline : {', '.join(f'`{n}`' for n in unknown)}")
        lines.append("")

    if previous:
        pcommit = (previous.get("run", {}).get("commit") or "?")[:7]
        lines.append(f"Comparé au run précédent `{pcommit}` ({previous.get('run', {}).get('date', '?')}).")
    else:
        lines.append("Aucun run précédent disponible pour la comparaison.")
    lines.append("")

    if problems:
        lines.append(f"**❌ {len(problems)} problème(s) :**")
        lines.extend(f"- {p}" for p in problems)
    else:
        lines.append("**✅ Aucune régression par rapport à la baseline.**")
    return problems, lines


def cmd_compare(args):
    results = json.loads(Path(args.results).read_text(encoding="utf-8"))
    baseline = json.loads(Path(args.baseline).read_text(encoding="utf-8"))
    previous = None
    if args.previous and Path(args.previous).is_file():
        try:
            previous = json.loads(Path(args.previous).read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            print(f"warning: ignoring unreadable previous results {args.previous}", file=sys.stderr)

    problems, lines = compare(results, baseline, previous)
    report = "\n".join(lines) + "\n"
    if args.report:
        Path(args.report).write_text(report, encoding="utf-8")
    print(report)
    return 1 if problems else 0


HISTORY_FIELDS = ["date", "commit", "branch", "status", "record",
                  "iterations", "errors", "used_before", "used_after", "cycles"]


def cmd_history(args):
    """Append one CSV row per RECORD of a run, to follow values over time."""
    import csv

    results = json.loads(Path(args.results).read_text(encoding="utf-8"))
    path = Path(args.csv)
    new_file = not path.exists()
    run = results.get("run", {})
    with path.open("a", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=HISTORY_FIELDS, extrasaction="ignore")
        if new_file:
            writer.writeheader()
        for rec in results.get("records", []):
            writer.writerow({
                "date": run.get("date"),
                "commit": (run.get("commit") or "")[:7],
                "branch": run.get("branch"),
                "status": results.get("status"),
                "record": rec["name"],
                **{k: rec.get(k) for k in HISTORY_FIELDS[5:]},
            })
    print(f"{len(results.get('records', []))} rows appended to {path}")
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)

    c = sub.add_parser("collect", help="parse a cargo test log into a results JSON file")
    c.add_argument("--log", required=True, help="cargo test output (stdout + stderr)")
    c.add_argument("--exit-code", type=int, required=True, help="exit code of cargo test")
    c.add_argument("--output", required=True)
    c.add_argument("--commit")
    c.add_argument("--branch")
    c.add_argument("--source", default="local", help="where the run happened (local, ci...)")
    c.set_defaults(func=cmd_collect)

    p = sub.add_parser("compare", help="check results against the baseline")
    p.add_argument("--results", required=True)
    p.add_argument("--baseline", required=True)
    p.add_argument("--previous", help="results of an earlier run, for the deltas")
    p.add_argument("--report", help="write the Markdown report to this file")
    p.set_defaults(func=cmd_compare)

    h = sub.add_parser("history", help="append a run's measurements to a CSV history")
    h.add_argument("--results", required=True)
    h.add_argument("--csv", required=True)
    h.set_defaults(func=cmd_history)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
