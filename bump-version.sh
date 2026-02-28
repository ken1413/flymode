#!/usr/bin/env bash
set -euo pipefail

# bump-version.sh — 統一更新 FlyMode 三個版本檔案
#
# Usage:
#   ./bump-version.sh 0.4.0
#   ./bump-version.sh patch   (0.3.0 → 0.3.1)
#   ./bump-version.sh minor   (0.3.1 → 0.4.0)
#   ./bump-version.sh major   (0.4.0 → 1.0.0)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CARGO_TOML="$SCRIPT_DIR/src-tauri/Cargo.toml"
TAURI_CONF="$SCRIPT_DIR/src-tauri/tauri.conf.json"
PACKAGE_JSON="$SCRIPT_DIR/src-ui/package.json"

# Read current version from Cargo.toml (single source of truth)
CURRENT=$(grep -m1 '^version' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CURRENT" ]; then
  echo "Error: cannot read current version from $CARGO_TOML"
  exit 1
fi

if [ $# -ne 1 ]; then
  echo "Current version: $CURRENT"
  echo ""
  echo "Usage: $0 <new-version|patch|minor|major>"
  echo ""
  echo "Examples:"
  echo "  $0 0.4.0    # set exact version"
  echo "  $0 patch    # $CURRENT → $(echo "$CURRENT" | awk -F. '{print $1"."$2"."$3+1}')"
  echo "  $0 minor    # $CURRENT → $(echo "$CURRENT" | awk -F. '{print $1"."$2+1".0"}')"
  echo "  $0 major    # $CURRENT → $(echo "$CURRENT" | awk -F. '{print $1+1".0.0"}')"
  exit 1
fi

ARG="$1"

# Calculate new version
case "$ARG" in
  patch)
    NEW=$(echo "$CURRENT" | awk -F. '{print $1"."$2"."$3+1}')
    ;;
  minor)
    NEW=$(echo "$CURRENT" | awk -F. '{print $1"."$2+1".0"}')
    ;;
  major)
    NEW=$(echo "$CURRENT" | awk -F. '{print $1+1".0.0"}')
    ;;
  *)
    # Validate semver format
    if ! echo "$ARG" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
      echo "Error: '$ARG' is not a valid semver (expected X.Y.Z)"
      exit 1
    fi
    NEW="$ARG"
    ;;
esac

if [ "$NEW" = "$CURRENT" ]; then
  echo "Already at version $CURRENT"
  exit 0
fi

echo "Bumping version: $CURRENT → $NEW"
echo ""

# 1. Cargo.toml
sed -i "0,/^version = \"$CURRENT\"/s//version = \"$NEW\"/" "$CARGO_TOML"
echo "  ✓ $CARGO_TOML"

# 2. tauri.conf.json
sed -i "s/\"version\": \"$CURRENT\"/\"version\": \"$NEW\"/" "$TAURI_CONF"
echo "  ✓ $TAURI_CONF"

# 3. package.json
sed -i "0,/\"version\": \"$CURRENT\"/s//\"version\": \"$NEW\"/" "$PACKAGE_JSON"
echo "  ✓ $PACKAGE_JSON"

echo ""
echo "Done! Version is now $NEW"
echo ""
echo "Next steps:"
echo "  cd src-tauri && cargo check        # verify Rust build"
echo "  cd src-ui && npm install            # update package-lock.json"
echo "  git add -A && git commit -m 'chore: bump version to $NEW'"
echo "  git tag v$NEW"
