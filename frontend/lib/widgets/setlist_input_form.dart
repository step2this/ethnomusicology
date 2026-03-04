import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config/constants.dart';
import '../providers/spotify_import_provider.dart';

class SetlistInputForm extends ConsumerStatefulWidget {
  final void Function({
    required String prompt,
    String? sourcePlaylistId,
    String? seedTracklist,
    required int trackCount,
    String? energyProfile,
    bool? creativeMode,
    double? bpmMin,
    double? bpmMax,
  }) onGenerate;

  const SetlistInputForm({super.key, required this.onGenerate});

  @override
  ConsumerState<SetlistInputForm> createState() => _SetlistInputFormState();
}

class _SetlistInputFormState extends ConsumerState<SetlistInputForm>
    with SingleTickerProviderStateMixin {
  late final TabController _tabController;
  final _promptController = TextEditingController();
  final _spotifyUrlController = TextEditingController();
  final _tracklistController = TextEditingController();
  final _bpmMinController = TextEditingController();
  final _bpmMaxController = TextEditingController();

  String? _selectedEnergyProfile;
  bool _creativeMode = false;
  double _trackCount = AppConstants.defaultTrackCount;
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
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildEnergyProfileSelector(),
          const SizedBox(height: 16),
          SwitchListTile(
            title: const Text('Creative Mode'),
            subtitle: const Text('Unexpected but compatible combinations'),
            value: _creativeMode,
            onChanged: (v) => setState(() => _creativeMode = v),
            contentPadding: EdgeInsets.zero,
          ),
          const SizedBox(height: 8),
          _buildTrackCountSlider(),
          const SizedBox(height: 8),
          _buildAdvancedSection(),
          const SizedBox(height: 16),
          TabBar(
            controller: _tabController,
            tabs: const [
              Tab(text: 'Describe a Vibe'),
              Tab(text: 'From Spotify'),
              Tab(text: 'From Tracklist'),
            ],
          ),
          const SizedBox(height: 16),
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
        Text('Energy Profile', style: Theme.of(context).textTheme.titleSmall),
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
            Text('Set Length', style: Theme.of(context).textTheme.titleSmall),
            const Spacer(),
            Text('${_trackCount.round()} tracks',
                style: Theme.of(context).textTheme.bodyMedium),
          ],
        ),
        Slider(
          value: _trackCount,
          min: AppConstants.minTrackCount,
          max: AppConstants.maxTrackCount,
          divisions: AppConstants.trackCountDivisions,
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
              Text('Advanced', style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(width: 4),
              Icon(
                _showAdvanced ? Icons.expand_less : Icons.expand_more,
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

  double? _parseBpm(String text) {
    if (text.trim().isEmpty) return null;
    return double.tryParse(text.trim());
  }

  void _generateFromVibe() {
    final prompt = _promptController.text.trim();
    if (prompt.isEmpty) return;
    widget.onGenerate(
      prompt: prompt,
      trackCount: _trackCount.round(),
      energyProfile: _selectedEnergyProfile,
      creativeMode: _creativeMode ? true : null,
      bpmMin: _parseBpm(_bpmMinController.text),
      bpmMax: _parseBpm(_bpmMaxController.text),
    );
  }

  Future<void> _generateFromSpotify() async {
    final url = _spotifyUrlController.text.trim();
    if (url.isEmpty) return;

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

      if (importId != null) {
        widget.onGenerate(
          prompt: 'Generate from imported playlist',
          sourcePlaylistId: importId,
          trackCount: _trackCount.round(),
          energyProfile: _selectedEnergyProfile,
          creativeMode: _creativeMode ? true : null,
          bpmMin: _parseBpm(_bpmMinController.text),
          bpmMax: _parseBpm(_bpmMaxController.text),
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
    widget.onGenerate(
      prompt: 'Generate from tracklist',
      seedTracklist: tracklist,
      trackCount: _trackCount.round(),
      energyProfile: _selectedEnergyProfile,
      creativeMode: _creativeMode ? true : null,
      bpmMin: _parseBpm(_bpmMinController.text),
      bpmMax: _parseBpm(_bpmMaxController.text),
    );
  }
}
