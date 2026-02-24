import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class AppTheme {
  static const _goldPrimary = Color(0xFFD4A843);
  static const _navySecondary = Color(0xFF1B2A4A);

  static ThemeData get lightTheme {
    final colorScheme = ColorScheme.fromSeed(
      seedColor: _goldPrimary,
      secondary: _navySecondary,
      brightness: Brightness.light,
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      textTheme: GoogleFonts.notoSansTextTheme(),
    );
  }

  static ThemeData get darkTheme {
    final colorScheme = ColorScheme.fromSeed(
      seedColor: _goldPrimary,
      secondary: _navySecondary,
      brightness: Brightness.dark,
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      textTheme: GoogleFonts.notoSansTextTheme(
        ThemeData(brightness: Brightness.dark).textTheme,
      ),
    );
  }
}
