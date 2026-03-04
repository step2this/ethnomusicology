import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../config/routes.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Tarab Studio'),
        centerTitle: true,
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text('Welcome to Tarab Studio'),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: () => context.go(AppRoutes.setlistGenerate),
              icon: const Icon(Icons.auto_awesome),
              label: const Text('Generate Setlist'),
            ),
            const SizedBox(height: 12),
            FilledButton.icon(
              onPressed: () => context.go(AppRoutes.trackCatalog),
              icon: const Icon(Icons.library_music),
              label: const Text('Track Catalog'),
            ),
            const SizedBox(height: 12),
            OutlinedButton.icon(
              onPressed: () => context.go(AppRoutes.spotifyImport),
              icon: const Icon(Icons.download),
              label: const Text('Import from Spotify'),
            ),
          ],
        ),
      ),
    );
  }
}
