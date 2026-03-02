import 'package:go_router/go_router.dart';

import '../screens/home_screen.dart';
import '../screens/setlist_generation_screen.dart';
import '../screens/spotify_import_screen.dart';
import '../screens/track_catalog_screen.dart';

final router = GoRouter(
  initialLocation: '/',
  routes: [
    GoRoute(
      path: '/',
      builder: (context, state) => const HomeScreen(),
    ),
    GoRoute(
      path: '/import/spotify',
      builder: (context, state) => const SpotifyImportScreen(),
    ),
    GoRoute(
      path: '/tracks',
      builder: (context, state) => const TrackCatalogScreen(),
    ),
    GoRoute(
      path: '/setlist/generate',
      builder: (context, state) => const SetlistGenerationScreen(),
    ),
  ],
);
