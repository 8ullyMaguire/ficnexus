#!/usr/bin/env bash
# PWA manifest sync check — verifies all referenced icons and screenshots exist.
# Run from repo root: ./scripts/check-pwa-manifest.sh

set -euo pipefail

MANIFEST="${1:-frontend/static/manifest.webmanifest}"
STATIC_DIR="$(dirname "$MANIFEST")"

if [[ ! -f "$MANIFEST" ]]; then
    echo "ERROR: manifest not found at $MANIFEST"
    exit 1
fi

echo "Checking PWA manifest: $MANIFEST"

ERRORS=0

# Extract all src values from JSON (simple grep-based extraction)
while IFS= read -r src; do
    # Strip quotes and leading slash
    clean="${src#\"}"
    clean="${clean%\"}"
    clean="${clean#/}"

    if [[ -z "$clean" ]]; then
        continue
    fi

    filepath="$STATIC_DIR/$clean"
    if [[ ! -f "$filepath" ]]; then
        echo "  MISSING: $src (expected at $filepath)"
        ((ERRORS++))
    else
        echo "  OK: $src"
    fi
done < <(grep -oP '"src"\s*:\s*"[^"]+' "$MANIFEST" | sed 's/"src"\s*:\s*//')

if [[ $ERRORS -gt 0 ]]; then
    echo ""
    echo "FAILED: $ERRORS missing file(s)"
    exit 1
fi

echo ""
echo "All PWA manifest references valid."
