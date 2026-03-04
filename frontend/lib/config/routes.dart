import 'package:go_router/go_router.dart';

import '../screens/home_screen.dart';
import '../screens/setlist_generation_screen.dart';
import '../screens/spotify_import_screen.dart';
import '../screens/track_catalog_screen.dart';

/// Centralized route path constants.
class AppRoutes {
  AppRoutes._();

  static const String home = '/';
  static const String spotifyImport = '/import/spotify';
  static const String trackCatalog = '/tracks';
  static const String setlistGenerate = '/setlist/generate';
}

final router = GoRouter(
  initialLocation: AppRoutes.home,
  routes: [
    GoRoute(
      path: AppRoutes.home,
      builder: (context, state) => const HomeScreen(),
    ),
    GoRoute(
      path: AppRoutes.spotifyImport,
      builder: (context, state) => const SpotifyImportScreen(),
    ),
    GoRoute(
      path: AppRoutes.trackCatalog,
      builder: (context, state) => const TrackCatalogScreen(),
    ),
    GoRoute(
      path: AppRoutes.setlistGenerate,
      builder: (context, state) => const SetlistGenerationScreen(),
    ),
  ],
);
