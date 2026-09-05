#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v tree-sitter >/dev/null 2>&1; then
    echo "error: tree-sitter CLI not found; install tree-sitter-cli@0.27.0" >&2
    echo "       (goldens are CLI version-sensitive; see harness/run.py)" >&2
    exit 2
fi

# Clones every pinned grammar (razor, c_sharp, xml) into harness/workspaces/.
echo "== snapshot harness (harness/run.py $*) =="
python3 harness/run.py "$@"

# The Razor grammar's own corpus lives in the grammar repository; run it
# against the same pinned checkout the harness just prepared.
RAZOR_WS="harness/workspaces/razor"
if [ -d "$RAZOR_WS/test/corpus" ]; then
    echo "== tree-sitter test ($RAZOR_WS, grammar corpus) =="
    (cd "$RAZOR_WS" && npm install --no-audit --no-fund --silent && tree-sitter test)
else
    echo "warning: $RAZOR_WS has no test/corpus; skipping grammar corpus run" >&2
fi
