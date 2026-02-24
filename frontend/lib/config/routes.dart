import 'package:go_router/go_router.dart';

import '../screens/home_screen.dart';
import '../screens/spotify_import_screen.dart';

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
  ],
);
