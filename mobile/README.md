# TijaraPOS Mobile

Flutter + GetX companion app that pairs with the Desktop POS over local
Wi-Fi. It never accesses SQLite directly — every read/write goes through
the Desktop's local API. See the repository root [`README.md`](../README.md)
and [`docs/architecture.md`](../docs/architecture.md) for the full picture.

## Commands

```bash
flutter pub get

flutter analyze
flutter test
flutter test integration_test   # from Phase 14 onward

flutter run                     # connected device/emulator
flutter build apk --release
flutter build ios --release     # requires macOS + Xcode
```

## Structure

```
lib/
├── core/       network, storage, theme, constants, utils
├── data/       models, services, repositories
├── modules/    auth, pairing, products, sales, customers, stock, profile
└── main.dart
```

See [`../docs/deployment.md`](../docs/deployment.md) for prerequisites and
environment notes.
