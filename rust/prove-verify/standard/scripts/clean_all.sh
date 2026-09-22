#!/usr/bin/env bash
# Delete local summaries, the Cargo build dir, and the Longfellow cmake tree.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "This will DELETE:"
echo "  - all local benchmark / measure artifact folders under standard/"
echo "  - target/ (the Rust build directory)"
echo "  - \$LONGFELLOW_ROOT/clang-build-release (rebuild required afterward)"
echo ""

read -r -p "Type 'delete' to confirm: " CONFIRM
if [[ "$CONFIRM" != "delete" ]]; then
  echo "Aborted."
  exit 1
fi

rm -rf \
  "$ROOT_DIR/prove-verify/artifacts_bench_prove_verify" \
  "$ROOT_DIR/prove-verify-no-cft/artifacts_bench_prove_verify_no_cft" \
  "$ROOT_DIR/prove-verify-revocation/artifacts_bench_prove_verify_revocation" \
  "$ROOT_DIR/merkle-vs-flat/artifacts_bench_merkle_vs_flat" \
  "$ROOT_DIR/communication-costs/artifacts_measure_communication_size" \
  "$ROOT_DIR/target"

# Longfellow is a vendored tree that is not duplicated here; resolve it the same
# way the benchmark harness does.
LF_ROOT="${LONGFELLOW_ROOT:-}"
if [[ -z "$LF_ROOT" ]]; then
  for candidate in \
    "$ROOT_DIR/longfellow-zk" \
    "$ROOT_DIR" \
    "$ROOT_DIR/../../../prove-verify/standard/longfellow-zk"; do
    if [[ -f "$candidate/lib/CMakeLists.txt" ]]; then
      LF_ROOT="$(cd "$candidate" && pwd)"
      break
    fi
  done
fi
if [[ -n "$LF_ROOT" && -d "$LF_ROOT/clang-build-release" ]]; then
  rm -rf "$LF_ROOT/clang-build-release"
  echo "Removed $LF_ROOT/clang-build-release."
fi

echo "Deleted artifact folders, target/ and clang-build-release."
