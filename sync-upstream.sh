#!/usr/bin/env bash
set -euo pipefail

# Merge the latest upstream SteelMC into this repo.
# Usage: ./sync-upstream.sh [branch]   (default: master)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

UPSTREAM_URL="https://github.com/Steel-Foundation/SteelMC.git"
BRANCH="${1:-master}"

if ! git remote get-url upstream &>/dev/null; then
    git remote add upstream "$UPSTREAM_URL"
fi

echo "Fetching upstream $BRANCH..."
if ! git fetch upstream "$BRANCH"; then
    echo "git fetch failed (network or git protocol issue)." >&2
    echo "Retry later, or sync manually:" >&2
    echo "  curl -L https://codeload.github.com/Steel-Foundation/SteelMC/tar.gz/refs/heads/$BRANCH -o /tmp/steelmc.tar.gz" >&2
    echo "  tar -xzf /tmp/steelmc.tar.gz --strip-components=1 -C .   # then review + commit" >&2
    exit 1
fi

echo "Merging upstream/$BRANCH..."
git merge "upstream/$BRANCH" --no-edit || {
    echo "" >&2
    echo "Merge conflicts — resolve them, then:" >&2
    echo "  git add -A && git commit" >&2
    echo "Checklist after every sync:" >&2
    echo "  - README.md stays AethelxSteelMC-branded" >&2
    echo "  - LICENSE stays AGPL-3.0-or-later" >&2
    echo "  - .gitignore keeps /steel-linux, .server.pid, and config/ tracked (no /config line)" >&2
    echo "  - .github/workflows keeps only test.yml, lint.yml, build.yml" >&2
    echo "  - rust-toolchain.toml pin matches .github/workflows/build.yml" >&2
    exit 1
}

echo "Done. Push to rebuild: git push"
