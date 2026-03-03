import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/models/setlist.dart';

void main() {
  group('BpmWarning', () {
    test('fromJson parses correctly', () {
      final json = {
        'from_position': 3,
        'to_position': 4,
        'bpm_delta': 8.5,
      };
      final warning = BpmWarning.fromJson(json);
      expect(warning.fromPosition, 3);
      expect(warning.toPosition, 4);
      expect(warning.bpmDelta, 8.5);
    });

    test('fromJson handles integer bpm_delta', () {
      final json = {
        'from_position': 1,
        'to_position': 2,
        'bpm_delta': 7,
      };
      final warning = BpmWarning.fromJson(json);
      expect(warning.bpmDelta, 7.0);
    });
  });

  group('Setlist.fromJson', () {
    Map<String, dynamic> fullJson() => {
          'id': 'set-1',
          'prompt': 'chill vibes',
          'model': 'claude-sonnet-4-20250514',
          'tracks': [
            {
              'position': 1,
              'title': 'Track A',
              'artist': 'Artist A',
              'original_position': 1,
              'source': 'catalog',
            },
          ],
          'energy_profile': 'warm-up',
          'catalog_percentage': 75.0,
          'catalog_warning': null,
          'bpm_warnings': [
            {
              'from_position': 1,
              'to_position': 2,
              'bpm_delta': 9.5,
            },
          ],
        };

    test('parses all new fields correctly', () {
      final setlist = Setlist.fromJson(fullJson());
      expect(setlist.energyProfile, 'warm-up');
      expect(setlist.catalogPercentage, 75.0);
      expect(setlist.catalogWarning, isNull);
      expect(setlist.bpmWarnings, hasLength(1));
      expect(setlist.bpmWarnings[0].fromPosition, 1);
      expect(setlist.bpmWarnings[0].toPosition, 2);
      expect(setlist.bpmWarnings[0].bpmDelta, 9.5);
    });

    test('handles missing new fields for backward compat', () {
      final json = {
        'id': 'set-2',
        'prompt': 'party set',
        'model': 'claude-sonnet-4-20250514',
        'tracks': [],
      };
      final setlist = Setlist.fromJson(json);
      expect(setlist.energyProfile, isNull);
      expect(setlist.catalogPercentage, isNull);
      expect(setlist.catalogWarning, isNull);
      expect(setlist.bpmWarnings, isEmpty);
    });

    test('parses catalog_warning when present', () {
      final json = fullJson();
      json['catalog_warning'] =
          'Low catalog match: only 25% of tracks from your library';
      json['catalog_percentage'] = 25.0;
      final setlist = Setlist.fromJson(json);
      expect(setlist.catalogWarning, contains('Low catalog match'));
      expect(setlist.catalogPercentage, 25.0);
    });

    test('parses multiple bpm_warnings', () {
      final json = fullJson();
      json['bpm_warnings'] = [
        {'from_position': 1, 'to_position': 2, 'bpm_delta': 8.0},
        {'from_position': 5, 'to_position': 6, 'bpm_delta': -7.5},
      ];
      final setlist = Setlist.fromJson(json);
      expect(setlist.bpmWarnings, hasLength(2));
      expect(setlist.bpmWarnings[1].bpmDelta, -7.5);
    });

    test('preserves existing fields alongside new ones', () {
      final json = fullJson();
      json['notes'] = 'Great flow';
      json['harmonic_flow_score'] = 85.0;
      json['score_breakdown'] = {
        'key_compatibility': 90.0,
        'bpm_continuity': 80.0,
        'energy_arc': 85.0,
      };
      json['created_at'] = '2026-03-02T12:00:00Z';
      final setlist = Setlist.fromJson(json);
      expect(setlist.notes, 'Great flow');
      expect(setlist.harmonicFlowScore, 85.0);
      expect(setlist.scoreBreakdown, isNotNull);
      expect(setlist.createdAt, '2026-03-02T12:00:00Z');
      // New fields still work
      expect(setlist.energyProfile, 'warm-up');
      expect(setlist.bpmWarnings, hasLength(1));
    });
  });
}
