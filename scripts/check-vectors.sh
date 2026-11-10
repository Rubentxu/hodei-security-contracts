#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cmp "$root/crates/hodei-hrn/tests/fixtures/vectors.json" "$root/contracts/hrn/vectors.json" \
  || { echo "contract vectors are out of sync; run scripts/sync-vectors.sh" >&2; exit 1; }
echo "contract vectors are in sync"
