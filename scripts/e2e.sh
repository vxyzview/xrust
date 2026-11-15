#!/usr/bin/env bash
# xrust end-to-end drill: builds nothing here — expects:
#   - PRoot v5.5.0 binary (built by build-proot.sh or provided)
#   - Alpine 3.24.4 minirootfs tarball (SHA-256 verified against dl-cdn)
# Runs the Rust e2e suite (PTY → parser → grid → real Alpine shell via PRoot)
# and an `apk update` network round-trip inside the rootfs.
set -euo pipefail

CACHE="${E2E_CACHE:-/tmp/buildcache}"
ROOTFS="$CACHE/rootfs"
PROOT="$CACHE/proot-src/src/proot"
REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "== 1. verify Alpine minirootfs SHA-256 against dl-cdn =="
cd "$CACHE"
curl -sS -o alpine.sha256 \
  https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/x86_64/alpine-minirootfs-3.24.4-x86_64.tar.gz.sha256
[ -f alpine-minirootfs-3.24.4-x86_64.tar.gz ] || \
  curl -sSL -o alpine-minirootfs-3.24.4-x86_64.tar.gz \
  https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/x86_64/alpine-minirootfs-3.24.4-x86_64.tar.gz
sha256sum -c alpine.sha256

echo "== 2. extract rootfs =="
rm -rf "$ROOTFS" && mkdir -p "$ROOTFS"
tar -xzf alpine-minirootfs-3.24.4-x86_64.tar.gz -C "$ROOTFS"
grep -q "3.24.4" "$ROOTFS/etc/alpine-release"

echo "== 3. Rust e2e suite (PTY + parser + grid + PRoot + Alpine) =="
cd "$REPO_DIR/xrust-core"
PROOT_BIN="$PROOT" ALPINE_ROOTFS="$ROOTFS" \
  CARGO_BUILD_JOBS=1 cargo test -j 1 --test e2e_proot -- --test-threads=1

echo "== 4. apk update inside the rootfs (real package database round-trip) =="
LD_LIBRARY_PATH="$CACHE/talloc-install/lib" "$PROOT" -R "$ROOTFS" -b /proc \
  /sbin/apk update --no-progress 2>&1 | tee /tmp/apk-update.log
grep -q "OK:" /tmp/apk-update.log && echo "APK_UPDATE_OK" || {
  echo "apk update did not report OK"; exit 1; }

echo "E2E_ALL_GREEN"
