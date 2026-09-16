#!/usr/bin/env bash
# Regenerate the committed TypeScript API bindings and verify they are current.
#
# Usage:
#   scripts/export-api-types.sh        Regenerate bindings in place (webapp/src/api/generated/)
#   scripts/export-api-types.sh check  Fail unless the committed bindings are already up to date
#
# The exporter is the api-exporter binary, which calls ts-rs programmatically.
# In check mode we regenerate into a temporary directory (via the ts-rs
# TS_RS_EXPORT_DIR env var) and compare against the committed tree, so the
# committed files are never clobbered.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMMITTED_DIR="$REPO_ROOT/webapp/src/api/generated"

run_exporter() {
    local out_dir="$1"
    # Run from the workspace root so the toolchain and workspace resolve consistently.
    (cd "$REPO_ROOT" && TS_RS_EXPORT_DIR="$out_dir" cargo run -p api-exporter)
}

cmd_export() {
    echo "Regenerating API type bindings..."
    run_exporter "$COMMITTED_DIR"
    echo "Done. Bindings are at $COMMITTED_DIR"
}

cmd_check() {
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    echo "Regenerating API type bindings into temporary directory for comparison..."
    run_exporter "$tmp_dir"

    if ! diff -r "$COMMITTED_DIR" "$tmp_dir" >/dev/null 2>&1; then
        echo "ERROR: committed TypeScript bindings are stale or nondeterministic." >&2
        echo "Run 'scripts/export-api-types.sh' to regenerate webapp/src/api/generated/." >&2
        diff -r "$COMMITTED_DIR" "$tmp_dir" >&2 || true
        exit 1
    fi

    echo "PASS: committed bindings are up to date and deterministic."
}

tmp_dir=""

case "${1:-export}" in
    export|regenerate) cmd_export ;;
    check)             cmd_check ;;
    *) echo "usage: $0 [export|check]" >&2; exit 2 ;;
esac
