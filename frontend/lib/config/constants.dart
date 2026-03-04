/// App-wide constants to avoid magic numbers scattered across the codebase.
class AppConstants {
  AppConstants._();

  // Setlist generation
  static const double defaultTrackCount = 15;
  static const double minTrackCount = 5;
  static const double maxTrackCount = 30;
  static const int trackCountDivisions = 25;

  // Catalog thresholds
  static const double lowCatalogThreshold = 30.0;
}
