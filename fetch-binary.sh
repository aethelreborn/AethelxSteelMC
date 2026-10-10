#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [[ -x ./steel-linux ]]; then
    echo "steel-linux already present ($(du -h ./steel-linux | cut -f1)). Remove it to re-download."
    exit 0
fi

# Prefer the CI artifact from this repo's latest green Build run.
if command -v gh &>/dev/null && git rev-parse --git-dir &>/dev/null; then
    run_id="$(gh run list --workflow=build.yml --limit 1 --status success \
        --json databaseId --jq '.[0].databaseId' 2>/dev/null || true)"
    if [[ -n "${run_id:-}" && "$run_id" != "null" ]]; then
        echo "Downloading steel-linux from CI run $run_id..."
        if gh run download "$run_id" -n steel-linux -D "$SCRIPT_DIR" 2>/dev/null; then
            chmod +x steel-linux
            echo "Done: ./steel-linux ($(du -h steel-linux | cut -f1))"
            exit 0
        fi
    fi
fi

echo "No CI artifact available — fetching latest upstream SteelMC release..."
curl -fL --retry 3 -o steel-linux \
    https://github.com/Steel-Foundation/SteelMC/releases/latest/download/steel-linux
chmod +x steel-linux
echo "Done: ./steel-linux ($(du -h steel-linux | cut -f1))"
