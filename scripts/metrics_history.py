import argparse
import csv
import io
import os
import re
import statistics

HEAP_SIZE = 100 * 1024
LONG_LIVED_BYTES = 4 * 64
CHURN_BLOCK = 1024
EXP1_ITERATIONS = 10000

FIELDS = [
    "date",
    "commit",
    "label",
    "run",
    "allocator",
    "i1_reuse_pct",
    "i1_iterations",
    "i2_live_blocks",
    "i3_cycles_per_iter",
    "exp1_errors",
    "exp2_errors",
    "exp3_errors",
    "exp4_frames_lost",
    "exp5_errors",
    "source_log",
]

RECORD = re.compile(
    r"RECORD name=(?P<name>\S+)\s+iterations=(?P<iterations>\d+)\s+"
    r"errors=(?P<errors>\d+)\s+used_before=(?P<used_before>\d+)\s+"
    r"used_after=(?P<used_after>\d+)\s+cycles=(?P<cycles>\d+)"
)
CAPACITY = re.compile(r"capacity per round:\s*([0-9 /]+?)\s*blocks")
FRAMES_LOST = re.compile(r"\(lost:\s*(\d+)\)")


def reuse_percent(iterations, used_after):
    freed = (iterations - 1) * CHURN_BLOCK
    if freed <= 0:
        return None
    resident = LONG_LIVED_BYTES + CHURN_BLOCK
    reused = freed - (used_after - resident)
    return round(max(0.0, min(100.0, reused * 100.0 / freed)), 2)


def parse(path):
    collected = {}
    capacities = []
    frames_lost = []
    with io.open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            match = RECORD.search(line)
            if match:
                name = match.group("name")
                collected.setdefault(name, []).append(
                    {
                        "iterations": int(match.group("iterations")),
                        "errors": int(match.group("errors")),
                        "used_after": int(match.group("used_after")),
                        "cycles": int(match.group("cycles")),
                    }
                )
                continue
            match = CAPACITY.search(line)
            if match:
                rounds = [int(v) for v in match.group(1).split("/") if v.strip()]
                capacities.append(min(rounds) if rounds else None)
                continue
            match = FRAMES_LOST.search(line)
            if match:
                frames_lost.append(int(match.group(1)))
    return collected, capacities, frames_lost


def nth(items, index, key=None):
    if index >= len(items):
        return ""
    value = items[index]
    return value if key is None else value[key]


def build_rows(path, date, commit, label, allocator):
    collected, capacities, frames_lost = parse(path)
    exp1 = collected.get("exp1_alloc_free_sequence", [])
    exp2 = collected.get("exp2_long_lived_vs_churn", [])
    exp3 = collected.get("exp3_exhaustion_and_recovery", [])
    exp5 = collected.get("exp5_under_interrupts", [])
    runs = max(len(exp1), len(exp2), len(exp3), len(exp5))
    rows = []
    for index in range(runs):
        reuse = ""
        iterations = ""
        if index < len(exp2):
            iterations = exp2[index]["iterations"]
            reuse = reuse_percent(iterations, exp2[index]["used_after"])
        cycles_per_iter = ""
        if index < len(exp1):
            cycles_per_iter = round(exp1[index]["cycles"] / EXP1_ITERATIONS)
        rows.append(
            {
                "date": date,
                "commit": commit,
                "label": label,
                "run": index + 1,
                "allocator": allocator,
                "i1_reuse_pct": reuse,
                "i1_iterations": iterations,
                "i2_live_blocks": nth(capacities, index),
                "i3_cycles_per_iter": cycles_per_iter,
                "exp1_errors": nth(exp1, index, "errors"),
                "exp2_errors": nth(exp2, index, "errors"),
                "exp3_errors": nth(exp3, index, "errors"),
                "exp4_frames_lost": nth(frames_lost, index),
                "exp5_errors": nth(exp5, index, "errors"),
                "source_log": os.path.basename(path),
            }
        )
    return rows


def append(history, rows):
    exists = os.path.exists(history)
    with io.open(history, "a", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=FIELDS)
        if not exists:
            writer.writeheader()
        for row in rows:
            writer.writerow(row)


def summarise(rows):
    cycles = [r["i3_cycles_per_iter"] for r in rows if r["i3_cycles_per_iter"] != ""]
    iterations = sorted({r["i1_iterations"] for r in rows if r["i1_iterations"] != ""})
    blocks = sorted({r["i2_live_blocks"] for r in rows if r["i2_live_blocks"] != ""})
    reuse = sorted({r["i1_reuse_pct"] for r in rows if r["i1_reuse_pct"] != ""})
    print("runs                : %d" % len(rows))
    print("I1 reuse (%%)        : %s" % (reuse or "n/a"))
    print("I1 iterations       : %s" % (iterations or "n/a"))
    print("I2 live blocks      : %s" % (blocks or "n/a"))
    if cycles:
        print(
            "I3 cycles/iteration : median %d, min %d, max %d"
            % (statistics.median(cycles), min(cycles), max(cycles))
        )


def main():
    parser = argparse.ArgumentParser(
        description="Extract objective D indicators from a kernel test log "
        "and append them to the indicator history."
    )
    parser.add_argument("log")
    parser.add_argument("--date", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--label", required=True)
    parser.add_argument("--allocator", default="bump")
    parser.add_argument(
        "--history",
        default=os.path.join("docs", "memoire", "metrics", "history.csv"),
    )
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    rows = build_rows(args.log, args.date, args.commit, args.label, args.allocator)
    if not rows:
        raise SystemExit("no RECORD line found in %s" % args.log)
    summarise(rows)
    if args.dry_run:
        return
    directory = os.path.dirname(args.history)
    if directory and not os.path.isdir(directory):
        os.makedirs(directory)
    append(args.history, rows)
    print("appended %d rows to %s" % (len(rows), args.history))


if __name__ == "__main__":
    main()
