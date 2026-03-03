import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist.dart';
import '../providers/setlist_provider.dart';
import '../providers/spotify_import_provider.dart';
import '../widgets/setlist_track_tile.dart';

class SetlistGenerationScreen extends ConsumerStatefulWidget {
  const SetlistGenerationScreen({super.key});

  @override
  ConsumerState<SetlistGenerationScreen> createState() =>
      _SetlistGenerationScreenState();
}

class _SetlistGenerationScreenState
    extends ConsumerState<SetlistGenerationScreen>
    with SingleTickerProviderStateMixin {
  late final TabController _tabController;
  final _promptController = TextEditingController();
  final _spotifyUrlController = TextEditingController();
  final _tracklistController = TextEditingController();
  final _bpmMinController = TextEditingController();
  final _bpmMaxController = TextEditingController();

  String? _selectedEnergyProfile;
  bool _creativeMode = false;
  double _trackCount = 15;
  bool _showAdvanced = false;
  bool _isImporting = false;
  String? _importError;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _promptController.dispose();
    _spotifyUrlController.dispose();
    _tracklistController.dispose();
    _bpmMinController.dispose();
    _bpmMaxController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(setlistProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Create Set'),
        centerTitle: true,
        actions: [
          if (state.hasSetlist)
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: _resetAll,
              tooltip: 'New setlist',
            ),
        ],
      ),
      body: state.hasSetlist || state.isLoading || state.error != null
          ? _buildContent(state)
          : _buildInputForm(state),
    );
  }

  Widget _buildInputForm(SetlistState state) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Energy profile selector
          _buildEnergyProfileSelector(),
          const SizedBox(height: 16),

          // Creative mode toggle
          SwitchListTile(
            title: const Text('Creative Mode'),
            subtitle:
                const Text('Unexpected but compatible combinations'),
            value: _creativeMode,
            onChanged: (v) => setState(() => _creativeMode = v),
            contentPadding: EdgeInsets.zero,
          ),
          const SizedBox(height: 8),

          // Set length slider
          _buildTrackCountSlider(),
          const SizedBox(height: 8),

          // Advanced BPM section
          _buildAdvancedSection(),
          const SizedBox(height: 16),

          // Tab bar for input modes
          TabBar(
            controller: _tabController,
            tabs: const [
              Tab(text: 'Describe a Vibe'),
              Tab(text: 'From Spotify'),
              Tab(text: 'From Tracklist'),
            ],
          ),
          const SizedBox(height: 16),

          // Tab content (fixed height to avoid layout issues in scroll)
          SizedBox(
            height: 200,
            child: TabBarView(
              controller: _tabController,
              children: [
                _buildVibeTab(),
                _buildSpotifyTab(),
                _buildTracklistTab(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEnergyProfileSelector() {
    const profiles = [
      ('warm-up', 'Warm-Up'),
      ('peak-time', 'Peak-Time'),
      ('journey', 'Journey'),
      ('steady', 'Steady'),
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Energy Profile',
            style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          children: profiles.map((p) {
            final (value, label) = p;
            return ChoiceChip(
              label: Text(label),
              selected: _selectedEnergyProfile == value,
              onSelected: (selected) {
                setState(() {
                  _selectedEnergyProfile = selected ? value : null;
                });
              },
            );
          }).toList(),
        ),
      ],
    );
  }

  Widget _buildTrackCountSlider() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text('Set Length',
                style: Theme.of(context).textTheme.titleSmall),
            const Spacer(),
            Text('${_trackCount.round()} tracks',
                style: Theme.of(context).textTheme.bodyMedium),
          ],
        ),
        Slider(
          value: _trackCount,
          min: 5,
          max: 30,
          divisions: 25,
          label: '${_trackCount.round()}',
          onChanged: (v) => setState(() => _trackCount = v),
        ),
      ],
    );
  }

  Widget _buildAdvancedSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => setState(() => _showAdvanced = !_showAdvanced),
          child: Row(
            children: [
              Text('Advanced',
                  style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(width: 4),
              Icon(
                _showAdvanced
                    ? Icons.expand_less
                    : Icons.expand_more,
                size: 20,
              ),
            ],
          ),
        ),
        if (_showAdvanced) ...[
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _bpmMinController,
                  decoration: const InputDecoration(
                    labelText: 'Min BPM',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: TextField(
                  controller: _bpmMaxController,
                  decoration: const InputDecoration(
                    labelText: 'Max BPM',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                ),
              ),
            ],
          ),
        ],
      ],
    );
  }

  Widget _buildVibeTab() {
    return Column(
      children: [
        TextField(
          controller: _promptController,
          decoration: const InputDecoration(
            hintText: 'Describe your ideal setlist...',
            border: OutlineInputBorder(),
            prefixIcon: Icon(Icons.music_note),
          ),
          maxLines: 3,
          minLines: 2,
          textInputAction: TextInputAction.send,
          onSubmitted: (_) => _generateFromVibe(),
        ),
        const SizedBox(height: 12),
        SizedBox(
          width: double.infinity,
          child: FilledButton(
            onPressed: _generateFromVibe,
            child: const Text('Generate'),
          ),
        ),
      ],
    );
  }

  Widget _buildSpotifyTab() {
    return Column(
      children: [
        TextField(
          controller: _spotifyUrlController,
          decoration: const InputDecoration(
            hintText: 'https://open.spotify.com/playlist/...',
            border: OutlineInputBorder(),
            prefixIcon: Icon(Icons.link),
            labelText: 'Spotify Playlist URL',
          ),
        ),
        const SizedBox(height: 12),
        if (_importError != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: Text(
              _importError!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
        SizedBox(
          width: double.infinity,
          child: _isImporting
              ? const Center(child: CircularProgressIndicator())
              : FilledButton.icon(
                  onPressed: _generateFromSpotify,
                  icon: const Icon(Icons.download),
                  label: const Text('Import & Generate'),
                ),
        ),
      ],
    );
  }

  Widget _buildTracklistTab() {
    return Column(
      children: [
        Expanded(
          child: TextField(
            controller: _tracklistController,
            decoration: const InputDecoration(
              hintText:
                  'Paste your tracklist here...\nArtist - Title\nArtist - Title',
              border: OutlineInputBorder(),
              alignLabelWithHint: true,
            ),
            maxLines: null,
            expands: true,
            textAlignVertical: TextAlignVertical.top,
          ),
        ),
        const SizedBox(height: 12),
        SizedBox(
          width: double.infinity,
          child: FilledButton(
            onPressed: _generateFromTracklist,
            child: const Text('Generate from Tracklist'),
          ),
        ),
      ],
    );
  }

  Widget _buildContent(SetlistState state) {
    if (state.error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.error_outline,
                size: 48, color: Theme.of(context).colorScheme.error),
            const SizedBox(height: 16),
            Text(state.error!, textAlign: TextAlign.center),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: _resetAll,
              child: const Text('Try Again'),
            ),
          ],
        ),
      );
    }

    if (state.isGenerating) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Generating your setlist...'),
          ],
        ),
      );
    }

    if (state.isArranging) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Arranging for harmonic flow...'),
          ],
        ),
      );
    }

    if (state.hasSetlist) {
      return _buildSetlistResult(state);
    }

    return const SizedBox.shrink();
  }

  Widget _buildSetlistResult(SetlistState state) {
    final setlist = state.setlist!;

    return Column(
      children: [
        // Notes
        if (setlist.notes != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              setlist.notes!,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    fontStyle: FontStyle.italic,
                  ),
            ),
          ),
        // Header row
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              Text(
                '${setlist.tracks.length} tracks',
                style: Theme.of(context).textTheme.labelLarge,
              ),
              if (setlist.isArranged) ...[
                const SizedBox(width: 12),
                Chip(
                  label:
                      Text('Flow: ${setlist.harmonicFlowScoreFormatted}'),
                  avatar: const Icon(Icons.auto_awesome, size: 16),
                ),
              ],
              // Catalog percentage badge
              if (setlist.catalogPercentage != null) ...[
                const SizedBox(width: 8),
                _buildCatalogBadge(setlist),
              ],
              const Spacer(),
              if (!setlist.isArranged)
                FilledButton.icon(
                  onPressed: () =>
                      ref.read(setlistProvider.notifier).arrangeSetlist(),
                  icon: const Icon(Icons.auto_awesome),
                  label: const Text('Arrange'),
                ),
            ],
          ),
        ),
        // Catalog warning
        if (setlist.catalogWarning != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Card(
              color: Theme.of(context).colorScheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: Row(
                  children: [
                    Icon(Icons.warning_amber,
                        size: 16,
                        color: Theme.of(context).colorScheme.error),
                    const SizedBox(width: 8),
                    Expanded(child: Text(setlist.catalogWarning!)),
                  ],
                ),
              ),
            ),
          ),
        // Track list
        Expanded(
          child: ListView.builder(
            itemCount: setlist.tracks.length,
            itemBuilder: (context, index) {
              final track = setlist.tracks[index];
              final hasBpmWarning = setlist.bpmWarnings.any((w) =>
                  w.fromPosition == track.position ||
                  w.toPosition == track.position);
              return SetlistTrackTile(
                track: track,
                hasBpmWarning: hasBpmWarning,
              );
            },
          ),
        ),
      ],
    );
  }

  Widget _buildCatalogBadge(Setlist setlist) {
    final pct = setlist.catalogPercentage!;
    final isLow = pct < 30;
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: isLow
            ? theme.colorScheme.errorContainer
            : theme.colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        'Catalog: ${pct.round()}%',
        style: theme.textTheme.labelSmall?.copyWith(
          color: isLow
              ? theme.colorScheme.onErrorContainer
              : theme.colorScheme.onPrimaryContainer,
        ),
      ),
    );
  }

  double? _parseBpm(String text) {
    if (text.trim().isEmpty) return null;
    return double.tryParse(text.trim());
  }

  void _generateFromVibe() {
    final prompt = _promptController.text.trim();
    if (prompt.isEmpty) return;
    _doGenerate(prompt);
  }

  Future<void> _generateFromSpotify() async {
    final url = _spotifyUrlController.text.trim();
    if (url.isEmpty) return;

    // Step 1: Import the playlist
    setState(() {
      _isImporting = true;
      _importError = null;
    });

    try {
      final importNotifier = ref.read(spotifyImportProvider.notifier);
      await importNotifier.importPlaylist(url);
      final importState = ref.read(spotifyImportProvider);

      if (importState.status == ImportStatus.error) {
        setState(() {
          _isImporting = false;
          _importError = importState.errorMessage ?? 'Import failed';
        });
        return;
      }

      final importId = importState.importId;
      setState(() => _isImporting = false);

      // Step 2: Generate from the imported playlist
      if (importId != null) {
        _doGenerate(
          'Generate from imported playlist',
          sourcePlaylistId: importId,
        );
      }
    } catch (e) {
      setState(() {
        _isImporting = false;
        _importError = e.toString();
      });
    }
  }

  void _generateFromTracklist() {
    final tracklist = _tracklistController.text.trim();
    if (tracklist.isEmpty) return;
    _doGenerate(
      'Generate from tracklist',
      seedTracklist: tracklist,
    );
  }

  void _doGenerate(
    String prompt, {
    String? sourcePlaylistId,
    String? seedTracklist,
  }) {
    ref.read(setlistProvider.notifier).generateSetlist(
          prompt,
          trackCount: _trackCount.round(),
          energyProfile: _selectedEnergyProfile,
          creativeMode: _creativeMode ? true : null,
          sourcePlaylistId: sourcePlaylistId,
          seedTracklist: seedTracklist,
          bpmMin: _parseBpm(_bpmMinController.text),
          bpmMax: _parseBpm(_bpmMaxController.text),
        );
  }

  void _resetAll() {
    ref.read(setlistProvider.notifier).reset();
    _promptController.clear();
    _spotifyUrlController.clear();
    _tracklistController.clear();
    _bpmMinController.clear();
    _bpmMaxController.clear();
    setState(() {
      _selectedEnergyProfile = null;
      _creativeMode = false;
      _trackCount = 15;
      _showAdvanced = false;
    });
  }
}
