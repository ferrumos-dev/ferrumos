#!/usr/bin/env bash
# Run the FerrumOS kernel tests under QEMU, keep the results and compare them
# with the baseline. Same command locally and in CI.
#
#   bash scripts/kernel_tests/run.sh [--previous <results.json>] [-- <extra cargo test args>]
#
# Output (in $OUT_DIR, default: kernel-test-results/ at the repository root):
#   kernel-tests.log   full cargo test output (build + serial output of every QEMU image)
#   results.json       parsed results: status of each image, RECORD measurements, environment
#   report.md          comparison with the baseline (and with --previous if given)
#
# Exit code: 0 when every test passes and no baseline rule is broken, 1 otherwise.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POC_DIR="${POC_DIR:-$ROOT/POC/ferrum_Poc_adaptative_sheduller}"
OUT_DIR="${OUT_DIR:-$ROOT/kernel-test-results}"
BASELINE="${BASELINE:-$POC_DIR/tests/baseline.json}"
SOURCE="${RESULTS_SOURCE:-local}"
PREVIOUS=""
CARGO_ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --previous) PREVIOUS="$2"; shift 2 ;;
        --) shift; CARGO_ARGS=("$@"); break ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

for tool in cargo bootimage qemu-system-x86_64 python3; do
    if ! command -v "$tool" >/dev/null; then
        echo "missing tool: $tool (see POC/ferrum_Poc_adaptative_sheduller/TEST_ENVIRONMENT.md)" >&2
        exit 2
    fi
done

mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/kernel-tests.log"

echo "==> cargo test in $POC_DIR (log: $LOG)"
# --no-fail-fast: one failing image must not hide the results of the others.
(cd "$POC_DIR" && cargo test --no-fail-fast "${CARGO_ARGS[@]}") 2>&1 | tee "$LOG"
CARGO_EXIT=${PIPESTATUS[0]}

python3 "$ROOT/scripts/kernel_tests/results.py" collect \
    --log "$LOG" \
    --exit-code "$CARGO_EXIT" \
    --output "$OUT_DIR/results.json" \
    --source "$SOURCE" \
    ${RESULTS_COMMIT:+--commit "$RESULTS_COMMIT"} \
    ${RESULTS_BRANCH:+--branch "$RESULTS_BRANCH"}

python3 "$ROOT/scripts/kernel_tests/results.py" compare \
    --results "$OUT_DIR/results.json" \
    --baseline "$BASELINE" \
    ${PREVIOUS:+--previous "$PREVIOUS"} \
    --report "$OUT_DIR/report.md"
