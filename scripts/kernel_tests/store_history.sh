#!/usr/bin/env bash
# CI only: commit the results of a run to the `test-results` branch, so they
# outlive the 90-day artifact retention and can be compared over time.
#
#   bash scripts/kernel_tests/store_history.sh <directory with results.json, report.md, kernel-tests.log>
#
# Layout of the test-results branch:
#   runs/<date>_<commit>/   results.json, report.md, kernel-tests.log of each run
#   history.csv             one row per measurement per run
#   latest.json             results of the last run that got past the build,
#                           used by the next runs as "previous" for the deltas
set -euo pipefail

SRC="$(realpath "$1")"
BRANCH="${RESULTS_BRANCH:-test-results}"
ROOT="$(git rev-parse --show-toplevel)"
SHA="${GITHUB_SHA:-$(git rev-parse HEAD)}"
WORK="$(mktemp -d)/history"

git config --global user.name "github-actions[bot]"
git config --global user.email "41898282+github-actions[bot]@users.noreply.github.com"

cleanup() { cd "$ROOT" && git worktree remove --force "$WORK" 2>/dev/null || true; }
trap cleanup EXIT

if git fetch --depth=1 origin "$BRANCH" 2>/dev/null; then
    git worktree add --detach "$WORK" FETCH_HEAD
else
    # first run: start the branch from nothing (no source code in it)
    git worktree add --orphan -b "results-tmp-$$" "$WORK"
fi

cd "$WORK"
if [[ ! -f README.md ]]; then
    cat > README.md <<'EOF'
# Résultats des tests noyau (QEMU)

Branche alimentée automatiquement par la CI (`.github/workflows/kernel-tests.yml`)
à chaque push sur `main`. Ne pas modifier à la main.

- `runs/<date>_<commit>/` : résultats complets de chaque exécution
- `history.csv` : une ligne par mesure et par exécution, pour suivre l'évolution
- `latest.json` : dernier run, sert de point de comparaison au run suivant

Voir `docs/greenlights/objecitf_obligatoire_2/md/automatisation_tests_qemu.md` sur la branche principale.
EOF
fi

RUN_DIR="runs/$(date -u +%Y-%m-%d_%H%M%S)_${SHA:0:7}"
mkdir -p "$RUN_DIR"
cp "$SRC"/* "$RUN_DIR"/

python3 "$ROOT/scripts/kernel_tests/results.py" history --results "$SRC/results.json" --csv history.csv
STATUS="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"])' "$SRC/results.json")"
if [[ "$STATUS" != "error" ]]; then
    cp "$SRC/results.json" latest.json
fi

git add -A
git commit -q -m "results ${SHA:0:7}: $STATUS"

for attempt in 1 2 3; do
    if git push origin "HEAD:$BRANCH"; then
        echo "results stored in $BRANCH/$RUN_DIR"
        exit 0
    fi
    echo "push rejected (attempt $attempt), rebasing on the remote branch"
    git pull --rebase origin "$BRANCH"
done
exit 1
