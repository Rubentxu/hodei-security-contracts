#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cp "$root/crates/hodei-hrn/tests/fixtures/vectors.json" "$root/contracts/hrn/vectors.json"
echo "synced contracts/hrn/vectors.json from the crate fixture"
