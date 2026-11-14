#!/usr/bin/env bash
# Bump firmware version, test, commit, tag, push, build, and flash OTA to the beacon.
#
# Usage:
#   scripts/release_ota.sh [-v X.Y.Z] [-m "changelog line"] [-H host] [--no-push] [--no-flash]
#
#   -v X.Y.Z     explicit version (default: next patch of beacon-firmware/Cargo.toml)
#   -m "text"    CHANGELOG "Changed" entry (default: "Maintenance release")
#   -H host      device address (default: timestation.local)
#   --no-push    skip git commit/tag/push
#   --no-flash   build only (binary left in /tmp)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

HOST="timestation.local"; VERSION=""; NOTE="Maintenance release"; PUSH=1; FLASH=1
while [ $# -gt 0 ]; do
  case "$1" in
    -v) VERSION="${2#v}"; shift 2;;
    -m) NOTE="$2"; shift 2;;
    -H) HOST="$2"; shift 2;;
    --no-push) PUSH=0; shift;;
    --no-flash) FLASH=0; shift;;
    *) echo "unknown arg $1"; exit 2;;
  esac
done

FW_TOML=firmware-rust/beacon-firmware/Cargo.toml
CUR=$(grep -m1 '^version' "$FW_TOML" | cut -d'"' -f2)
if [ -z "$VERSION" ]; then
  IFS=. read -r MA MI PA <<<"$CUR"; VERSION="$MA.$MI.$((PA+1))"
fi
echo "==> $CUR -> $VERSION"

# 1. Version bump everywhere
sed -i '' "1,5s/^version = \".*\"/version = \"$VERSION\"/" "$FW_TOML" firmware-rust/beacon-core/Cargo.toml
sed -i '' "s/id=\"versionBadge\">v[0-9.]*</id=\"versionBadge\">v$VERSION</; s/id=\"otaFirmwareVersion\">v[0-9.]*</id=\"otaFirmwareVersion\">v$VERSION</" data/index.html
sed -i '' "s/currentFirmwareVersion = '[0-9.]*'/currentFirmwareVersion = '$VERSION'/" data/app.js
sed -i '' "s/\"version\": \"v[0-9.]*\"/\"version\": \"v$VERSION\"/" docs/manifest.json
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
  python3 - "$VERSION" "$NOTE" <<'PY'
import sys, datetime
v, note = sys.argv[1], sys.argv[2]
p = "CHANGELOG.md"; s = open(p).read()
entry = f"## [{v}] - {datetime.date.today()}\n\n### Changed\n- {note}\n\n"
i = s.index("## [")
open(p, "w").write(s[:i] + entry + s[i:])
PY
fi

# 2. Tests
node --check data/app.js
(cd firmware-rust && cargo test --package beacon-core --target aarch64-apple-darwin)

# 3. Build OTA & merged factory images (also updates docs/ for GitHub Pages web installer)
(cd firmware-rust/beacon-firmware && cargo build --release)
BIN="/tmp/multiband6-beacon-$VERSION.bin"
espflash save-image --chip esp32c3 \
  --partition-table firmware-rust/beacon-firmware/partitions.csv \
  firmware-rust/target/riscv32imc-esp-espidf/release/beacon-firmware "$BIN"
espflash save-image --merge --skip-padding --chip esp32c3 \
  --partition-table firmware-rust/beacon-firmware/partitions.csv \
  firmware-rust/target/riscv32imc-esp-espidf/release/beacon-firmware \
  docs/multiband6-beacon-factory.bin
ls -lh "$BIN" docs/multiband6-beacon-factory.bin

# 4. Commit, tag, push (push triggers the GitHub release workflow)
if [ "$PUSH" = 1 ]; then
  git add -A
  git commit -m "chore(release): bump firmware to v$VERSION"
  git tag "v$VERSION"
  git push origin main
  git push origin "v$VERSION"
fi

[ "$FLASH" = 1 ] || { echo "Built $BIN"; exit 0; }

# 5. OTA flash (retry: the device drops connections on weak Wi-Fi)
for i in 1 2 3; do
  echo "==> OTA upload attempt $i to $HOST"
  if curl -sS -m 180 -X POST -H "Content-Type: application/octet-stream" \
       --data-binary @"$BIN" "http://$HOST/api/ota/upload" | grep -q flashed; then OK=1; break; fi
  sleep 3
done
[ "${OK:-0}" = 1 ] || { echo "OTA failed; binary kept at $BIN"; exit 1; }

# 6. Verify reboot into new version
for i in $(seq 1 20); do
  sleep 3
  V=$(curl -s -m 4 "http://$HOST/api/status" | python3 -c 'import sys,json;print(json.load(sys.stdin)["version"])' 2>/dev/null || true)
  [ "$V" = "$VERSION" ] && { echo "==> Device running v$V"; rm -f "$BIN"; exit 0; }
done
echo "Device did not report v$VERSION (rollback protection may have reverted it)"; exit 1
