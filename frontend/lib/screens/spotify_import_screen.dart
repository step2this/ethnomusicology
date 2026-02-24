import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/spotify_import_provider.dart';
import '../widgets/import_progress.dart';
import '../widgets/import_summary.dart';

class SpotifyImportScreen extends ConsumerStatefulWidget {
  const SpotifyImportScreen({super.key});

  @override
  ConsumerState<SpotifyImportScreen> createState() =>
      _SpotifyImportScreenState();
}

class _SpotifyImportScreenState extends ConsumerState<SpotifyImportScreen> {
  final _urlController = TextEditingController();
  final _formKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    // Check connection status on load
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(spotifyConnectionProvider.notifier).checkConnection('dev-user');
    });
  }

  @override
  void dispose() {
    _urlController.dispose();
    super.dispose();
  }

  String? _validatePlaylistUrl(String? value) {
    if (value == null || value.trim().isEmpty) {
      return 'Please enter a Spotify playlist URL or URI';
    }
    final trimmed = value.trim();
    final urlPattern = RegExp(
      r'^https://open\.spotify\.com/playlist/[a-zA-Z0-9]+(\?.*)?$',
    );
    final uriPattern = RegExp(r'^spotify:playlist:[a-zA-Z0-9]+$');
    if (!urlPattern.hasMatch(trimmed) && !uriPattern.hasMatch(trimmed)) {
      return 'Expected: https://open.spotify.com/playlist/... or spotify:playlist:...';
    }
    return null;
  }

  void _startImport() {
    if (!_formKey.currentState!.validate()) return;
    ref
        .read(spotifyImportProvider.notifier)
        .importPlaylist(_urlController.text.trim());
  }

  @override
  Widget build(BuildContext context) {
    final connectionState = ref.watch(spotifyConnectionProvider);
    final importState = ref.watch(spotifyImportProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Import from Spotify'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Connection status card
            _ConnectionCard(
              state: connectionState,
              onConnect: () async {
                final messenger = ScaffoldMessenger.of(context);
                final url = await ref
                    .read(spotifyConnectionProvider.notifier)
                    .getAuthorizationUrl('dev-user');
                if (url != null && mounted) {
                  messenger.showSnackBar(
                    SnackBar(
                      content: Text('Open this URL to connect: $url'),
                      duration: const Duration(seconds: 10),
                    ),
                  );
                }
              },
            ),

            const SizedBox(height: 24),

            // Import form (only shown when connected)
            if (connectionState.status ==
                SpotifyConnectionStatus.connected) ...[
              Text(
                'Paste a Spotify playlist URL or URI',
                style: theme.textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              Form(
                key: _formKey,
                child: TextFormField(
                  controller: _urlController,
                  validator: _validatePlaylistUrl,
                  decoration: const InputDecoration(
                    hintText:
                        'https://open.spotify.com/playlist/... or spotify:playlist:...',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.link),
                  ),
                  enabled: importState.status != ImportStatus.importing,
                ),
              ),
              const SizedBox(height: 16),
              FilledButton.icon(
                onPressed: importState.status == ImportStatus.importing
                    ? null
                    : _startImport,
                icon: const Icon(Icons.download),
                label: const Text('Import Playlist'),
              ),
            ],

            const SizedBox(height: 24),

            // Import progress / summary / error
            if (importState.status == ImportStatus.importing)
              const ImportProgressWidget(),
            if (importState.status == ImportStatus.completed)
              ImportSummaryCard(progress: importState.progress),
            if (importState.status == ImportStatus.error)
              _ErrorCard(
                message: importState.errorMessage ?? 'Unknown error',
                onRetry: () {
                  ref.read(spotifyImportProvider.notifier).reset();
                },
              ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Connection card
// ---------------------------------------------------------------------------

class _ConnectionCard extends StatelessWidget {
  final SpotifyConnectionState state;
  final VoidCallback onConnect;

  const _ConnectionCard({required this.state, required this.onConnect});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(
              state.status == SpotifyConnectionStatus.connected
                  ? Icons.check_circle
                  : Icons.cloud_off,
              color: state.status == SpotifyConnectionStatus.connected
                  ? Colors.green
                  : theme.colorScheme.onSurfaceVariant,
              size: 32,
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Spotify Connection',
                    style: theme.textTheme.titleSmall,
                  ),
                  Text(
                    _statusText,
                    style: theme.textTheme.bodySmall,
                  ),
                ],
              ),
            ),
            if (state.status != SpotifyConnectionStatus.connected)
              FilledButton.tonal(
                onPressed:
                    state.status == SpotifyConnectionStatus.connecting
                        ? null
                        : onConnect,
                child: state.status == SpotifyConnectionStatus.connecting
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Text('Connect'),
              ),
          ],
        ),
      ),
    );
  }

  String get _statusText {
    switch (state.status) {
      case SpotifyConnectionStatus.disconnected:
        return 'Not connected. Tap Connect to authorize.';
      case SpotifyConnectionStatus.connecting:
        return 'Connecting...';
      case SpotifyConnectionStatus.connected:
        return 'Connected and ready to import.';
      case SpotifyConnectionStatus.error:
        return state.errorMessage ?? 'Connection failed.';
    }
  }
}

// ---------------------------------------------------------------------------
// Error card
// ---------------------------------------------------------------------------

class _ErrorCard extends StatelessWidget {
  final String message;
  final VoidCallback onRetry;

  const _ErrorCard({required this.message, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.error_outline,
                    color: Theme.of(context).colorScheme.error),
                const SizedBox(width: 8),
                Text(
                  'Import Failed',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                        color: Theme.of(context).colorScheme.error,
                      ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(message),
            const SizedBox(height: 12),
            FilledButton.tonal(
              onPressed: onRetry,
              child: const Text('Try Again'),
            ),
          ],
        ),
      ),
    );
  }
}
