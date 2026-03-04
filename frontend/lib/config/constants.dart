/// App-wide constants to avoid magic numbers scattered across the codebase.
class AppConstants {
  AppConstants._();

  // Audio / crossfade
  static const double defaultCrossfadeDuration = 4.0;
  static const double minCrossfadeDuration = 1.0;
  static const double maxCrossfadeDuration = 8.0;
  static const int crossfadeDivisions = 7;
  static const double crossfadeSoloSeconds = 2.0;

  // Setlist generation
  static const double defaultTrackCount = 15;
  static const double minTrackCount = 5;
  static const double maxTrackCount = 30;
  static const int trackCountDivisions = 25;

  // Catalog thresholds
  static const double lowCatalogThreshold = 30.0;
}
