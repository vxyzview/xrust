#!/usr/bin/env bash
# Build the Rust cdylib for Android ABIs into app/src/main/jniLibs/.
# Requires: cargo ndk (cargo install cargo-ndk), ANDROID_NDK_HOME set.
# Usage: scripts/build-android-rust.sh [--abi arm64-v8a]
set -euo pipefail

cd "$(dirname "$0")/../xrust-core"

OUT="$(cd ../app/src/main/jniLibs && pwd)"
ABIS=(arm64-v8a armeabi-v7a x86_64 x86)
if [[ "${1:-}" == "--abi" ]]; then
  ABIS=("$2")
fi

for ABI in "${ABIS[@]}"; do
  echo "== cargo ndk for $ABI =="
  cargo ndk -t "$ABI" -o "$OUT" build --release
done

ls -la "$OUT"/*/libxrust_core.so
