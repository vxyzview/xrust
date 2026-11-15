#!/usr/bin/env bash
# Build PRoot v5.5.0 (proot-me) for an Android ABI using the NDK toolchain,
# and copy the result into app assets (packaged as libproot.so to keep the
# exec bit through APK packaging).
# Usage: scripts/build-proot.sh [arm64-v8a|armeabi-v7a|x86_64|x86] [ndk-triple]
set -euo pipefail

VERSION=5.5.0
CACHE="${E2E_CACHE:-/tmp/buildcache}"
ABI="${1:-arm64-v8a}"

case "$ABI" in
  arm64-v8a) TRIPLE="${2:-aarch64-linux-android21}" ;;
  armeabi-v7a) TRIPLE="${2:-armv7a-linux-androideabi21}" ;;
  x86_64) TRIPLE="${2:-x86_64-linux-android21}" ;;
  x86) TRIPLE="${2:-i686-linux-android21}" ;;
  *) echo "unknown abi $ABI"; exit 1 ;;
esac

CC="${CC:-${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/${TRIPLE}-clang}"
[ -x "$CC" ] || { echo "NDK clang not found at $CC (set ANDROID_NDK_HOME)"; exit 1; }

SRC="$CACHE/proot-src"
if [ ! -d "$SRC" ]; then
  curl -sSL "https://github.com/proot-me/proot/archive/refs/tags/v${VERSION}.tar.gz" -o "$CACHE/proot.tar.gz"
  tar -xzf "$CACHE/proot.tar.gz" -C "$CACHE"
  mv "$CACHE/proot-${VERSION}" "$SRC"
fi

# PRoot needs talloc; build a minimal local copy when pkg-config has none.
if ! pkg-config --exists talloc; then
  echo "building local talloc…"
  TA="$CACHE/talloc-2.4.2"
  [ -d "$TA" ] || { curl -sSL https://www.samba.org/ftp/talloc/talloc-2.4.2.tar.gz -o "$CACHE/talloc.tar.gz"; tar -xzf "$CACHE/talloc.tar.gz" -C "$CACHE"; }
  (cd "$TA" && ./configure --prefix="$CACHE/talloc-install" --disable-python --without-gettext >/dev/null && make >/dev/null && make install >/dev/null)
  export PKG_CONFIG_PATH="$CACHE/talloc-install/lib/pkgconfig"
fi

make -C "$SRC/src" clean >/dev/null 2>&1 || true
CC="$CC" make -C "$SRC/src" -j"$(nproc)" LDFLAGS="-Wl,-z,noexecstack -static"

OUT="$(dirname "$0")/../app/src/main/assets"
mkdir -p "$OUT"
cp "$SRC/src/proot" "$OUT/proot-$ABI"
echo "built $OUT/proot-$ABI"
