# Development Environment & Deployment

## Prerequisites

### Desktop (`desktop/`)
- Node.js 20+ and npm
- Rust stable (via `rustup`) + Cargo
- Linux only: Tauri's system dependencies —
  `libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev
  libssl-dev libsecret-1-dev build-essential`
  (Windows/macOS have their own prerequisites — see the
  [Tauri prerequisites guide](https://tauri.app/start/prerequisites/))

### Mobile (`mobile/`)
- Flutter 3.x (stable channel) + Dart SDK (bundled with Flutter)
- Android Studio/SDK for Android builds, Xcode for iOS builds
- Linux desktop target additionally needs `libsecret-1-dev` (used by
  `flutter_secure_storage`) if you build/run the Linux target for local
  testing

## Desktop: setup & commands

```bash
cd desktop
npm install

npm run dev          # Vite dev server only (UI iteration, no native shell)
npm run tauri dev    # Full app: Rust backend + webview, hot-reloading UI

npm run typecheck    # tsc project references, no emit
npm run lint         # oxlint
npm run format       # prettier --write
npm run build        # tsc -b && vite build -> dist/
npm run tauri build  # Produces a native installer/bundle for the current OS
```

Rust-only checks (from `desktop/src-tauri/`): `cargo check`, `cargo test`
(tests land starting Phase 2).

## Mobile: setup & commands

```bash
cd mobile
flutter pub get

flutter analyze      # Static analysis
flutter test         # Unit/widget tests
flutter test integration_test  # End-to-end tests (from Phase 14 onward)
flutter format .     # or: dart format .

flutter run                       # Run on a connected device/emulator
flutter run -d linux              # Linux desktop target, useful for quick UI checks
flutter build apk --release       # Android
flutter build ios --release       # iOS (requires macOS + Xcode)
```

To pair against a Desktop instance during development, run both apps on
the same Wi-Fi network/subnet; the Desktop UI displays its LAN IP, port,
and pairing code once the API server exists (Phase 10-11).

## Known limitations of this development container

These are limitations of the sandbox this repository was scaffolded in,
not of the project itself:

- No Android SDK or Xcode is installed here, so mobile Android/iOS builds
  are unverified in this environment — `flutter analyze`, `flutter test`,
  and `flutter build linux` were used instead to validate the toolchain
  and code. Run `flutter build apk`/`flutter build ios` on a machine with
  the respective SDK before shipping to a device.
- Only the Linux Tauri target was built/verified here (`cargo check`
  inside `desktop/src-tauri`). Windows/macOS bundles need to be built on
  those platforms (or via CI with the matching runners).
- `npx shadcn add <component>` could not reach `ui.shadcn.com` from this
  container's network policy; components are hand-authored following the
  same conventions (`components.json` is configured so the CLI works
  normally in an environment with unrestricted network access).

## Backup & restore (Phase 17)

Not yet implemented. Once built: `Create Backup`/`Restore Backup` will
copy the SQLite file (or use SQLite's online backup API) after validating
integrity, taking a safety snapshot of the current database before ever
overwriting it, and restoring atomically (write to a temp path, verify,
then swap).

## Production packaging (Phase 20)

Not yet implemented. Planned: `tauri build` produces the platform
installer for Desktop; Flutter's standard `build apk`/`build appbundle`/
`build ios` produce the Mobile artifacts. CI matrix and code-signing setup
land in Phase 20.
