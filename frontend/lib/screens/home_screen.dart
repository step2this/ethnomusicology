import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Salamic Vibes'),
        centerTitle: true,
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text('Welcome to Salamic Vibes'),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: () => context.go('/import/spotify'),
              icon: const Icon(Icons.download),
              label: const Text('Import from Spotify'),
            ),
          ],
        ),
      ),
    );
  }
}
