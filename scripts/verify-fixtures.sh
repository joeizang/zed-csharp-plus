#!/usr/bin/env bash
# verify-fixtures.sh — build verification for the M0.3b fixture solutions.
#
# For each fixture: ensure a restore has run once (creating the restore
# marker), then build with --no-restore. The --no-restore pass approximates
# the offline-after-first-restore guarantee: no package resolution is needed
# once the first restore has populated obj/ and the NuGet cache.
#
# Prints a compact pass/fail table and exits non-zero if anything fails.
#
# Usage: scripts/verify-fixtures.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/../fixtures"

targets=(
  "mvc-web/MvcWeb.csproj"
  "razor-pages/RazorPages.csproj"
  "blazor-webapp/BlazorWebApp.csproj"
  "blazor-wasm/BlazorWasm.csproj"
  "razor-classlib/RazorClassLib.csproj"
  "multi-project/MultiProject.slnx"
)

# Restore marker check: a project is restored when obj/project.assets.json
# exists next to it. For a .slnx, every referenced project must be restored.
is_restored() {
  local target="$1" dir proj
  dir="$(dirname "$target")"
  if [[ "$target" == *.slnx ]]; then
    while IFS= read -r proj; do
      [[ -f "$dir/$proj" && -f "$(dirname "$dir/$proj")/obj/project.assets.json" ]] || return 1
    done < <(sed -n 's/.*Path="\([^"]*\)".*/\1/p' "$target")
  else
    [[ -f "$dir/obj/project.assets.json" ]] || return 1
  fi
}

run_dotnet() {
  local log
  log="$(mktemp)"
  if dotnet "$@" >"$log" 2>&1; then
    rm -f "$log"
    return 0
  fi
  echo "--- 'dotnet $*' failed; last lines: ---" >&2
  tail -15 "$log" >&2
  rm -f "$log"
  return 1
}

fail=0
printf "%-16s %-36s %-8s %s\n" "FIXTURE" "TARGET" "RESTORE" "BUILD"
for rel in "${targets[@]}"; do
  name="${rel%%/*}"
  target="$FIXTURES_DIR/$rel"
  restore="ok"
  build="ok"

  if [[ ! -f "$target" ]]; then
    printf "%-16s %-36s %-8s %s\n" "$name" "$rel" "n/a" "MISSING"
    fail=1
    continue
  fi

  if ! is_restored "$target"; then
    if ! run_dotnet restore "$target"; then
      restore="FAIL"
    fi
  fi

  if [[ "$restore" == "FAIL" ]]; then
    build="FAIL"
  elif ! run_dotnet build "$target" --no-restore; then
    build="FAIL"
  fi

  [[ "$restore" == "FAIL" || "$build" == "FAIL" ]] && fail=1
  printf "%-16s %-36s %-8s %s\n" "$name" "$rel" "$restore" "$build"
done

if [[ "$fail" -ne 0 ]]; then
  echo "verify-fixtures: FAIL" >&2
else
  echo "verify-fixtures: all fixtures build offline after first restore"
fi
exit "$fail"
