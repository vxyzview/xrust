# Release checklist

## Prerelease (v1.0.0-alpha)

- [ ] CI green on `main` (Rust fmt/clippy/test + gradle assembleDebug).
- [ ] `./scripts/e2e.sh` green locally (PRoot + Alpine 3.24.4 + apk update).
- [ ] Manual smoke on device/emulator: create session → type → colors →
      resize → switch apps → return (reader resumes) → exit kills children.
- [ ] `connectedAndroidTest` passes (NativeSmokeTest).

## Signing

- [ ] Create keystore: `keytool -genkey -v -keystore release.jks -keyalg RSA
      -keysize 4096 -validity 10000 -alias xrust`.
- [ ] Repository Settings → Environments → `release-signing` secrets:
      `ANDROID_KEYSTORE_BASE64` (base64 of the jks), `ANDROID_KEYSTORE_PASSWORD`,
      `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`.
- [ ] `docs/RELEASE_NOTES.md` updated for the tag.

## Tag & ship

- [ ] `git tag v1.0.0-alpha && git push --tags` → release.yml builds a
      **draft** prerelease with signed AAB + APKs.
- [ ] Verify artifact contents: AAB installs via bundletool, APKs install
      per-ABI.
- [ ] Promote to stable: new tag `v1.0.0` (draft=false, prerelease=false).

## Rollback plan

Releases are immutable artifacts; roll back by shipping `vX.(Y-1).Z` as a new
tag. The rootfs format is stable across patch versions (Alpine 3.24.x).
