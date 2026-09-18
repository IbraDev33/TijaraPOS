import 'package:flutter/material.dart';

/// Central theme definition so no screen hardcodes colors or text styles.
/// Populated further as modules are built; kept minimal for the Phase 1
/// scaffold.
abstract final class AppTheme {
  static const Color _seed = Color(0xFF1D4ED8);

  static ThemeData get light => ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(seedColor: _seed),
  );

  static ThemeData get dark => ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: _seed,
      brightness: Brightness.dark,
    ),
  );
}
