#!/usr/bin/env bash
# Project-reload case (M0.3b): adds a scratch Razor Page to fixtures/razor-pages
# so project reload can be observed while an editor session / server is running.
#
# Safe and idempotent: refuses to overwrite existing files and exits non-zero
# if the target page already exists. Delete the two generated files (listed in
# README.md) to reset the case.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAGES_DIR="$SCRIPT_DIR/../razor-pages/Pages"
PAGE_NAME="ReloadProbe"
NAMESPACE="RazorPages.Pages"

targets=(
  "$PAGES_DIR/$PAGE_NAME.cshtml"
  "$PAGES_DIR/$PAGE_NAME.cshtml.cs"
)

for target in "${targets[@]}"; do
  if [[ -e "$target" ]]; then
    echo "error: $target already exists; refusing to overwrite." >&2
    echo "Delete the ReloadProbe files (see README.md) to re-run this case." >&2
    exit 1
  fi
done

if [[ ! -f "$SCRIPT_DIR/../razor-pages/RazorPages.csproj" ]]; then
  echo "error: fixtures/razor-pages not found next to this script." >&2
  exit 1
fi

dotnet new page -n "$PAGE_NAME" -o "$PAGES_DIR" --namespace "$NAMESPACE"

echo "Added:"
for target in "${targets[@]}"; do
  echo "  $target"
done
echo
echo "The page is picked up automatically (SDK-style globbing); no .csproj edit."
echo "Observe: dotnet watch rebuild, and the editor reloading the project."
