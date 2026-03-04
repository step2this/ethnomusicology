import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/models/track.dart';

void main() {
  group('Track.fromJson', () {
    test('parses valid track JSON', () {
      final json = {
        'id': 'track-1',
        'title': 'Desert Rose',
        'artist': 'Sting',
        'album': 'Brand New Day',
        'duration_ms': 285000,
        'bpm': 102.5,
        'key': 'Am',
        'energy': 0.65,
        'source': 'spotify',
        'source_id': 'sp-123',
        'preview_url': 'https://example.com/preview.mp3',
        'album_art_url': 'https://example.com/art.jpg',
        'date_added': '2026-03-01T12:00:00Z',
      };

      final track = Track.fromJson(json);

      expect(track.id, 'track-1');
      expect(track.title, 'Desert Rose');
      expect(track.artist, 'Sting');
      expect(track.album, 'Brand New Day');
      expect(track.durationMs, 285000);
      expect(track.bpm, 102.5);
      expect(track.key, 'Am');
      expect(track.energy, 0.65);
      expect(track.source, 'spotify');
      expect(track.sourceId, 'sp-123');
      expect(track.previewUrl, 'https://example.com/preview.mp3');
      expect(track.albumArtUrl, 'https://example.com/art.jpg');
      expect(track.dateAdded, isA<DateTime>());
    });

    test('handles missing optional fields', () {
      final json = {
        'id': 'track-2',
        'title': 'Minimal Track',
        'artist': 'Unknown',
        'source': 'spotify',
        'date_added': '2026-03-01T00:00:00Z',
      };

      final track = Track.fromJson(json);

      expect(track.id, 'track-2');
      expect(track.title, 'Minimal Track');
      expect(track.album, isNull);
      expect(track.durationMs, isNull);
      expect(track.bpm, isNull);
      expect(track.key, isNull);
      expect(track.energy, isNull);
      expect(track.sourceId, isNull);
      expect(track.previewUrl, isNull);
      expect(track.albumArtUrl, isNull);
    });

    test('defaults source to spotify when missing', () {
      final json = {
        'id': 'track-3',
        'title': 'No Source',
        'artist': 'Artist',
        'date_added': '2026-03-01T00:00:00Z',
      };

      final track = Track.fromJson(json);
      expect(track.source, 'spotify');
    });

    test('parses integer bpm as double', () {
      final json = {
        'id': 'track-4',
        'title': 'Int BPM',
        'artist': 'Artist',
        'source': 'spotify',
        'bpm': 128, // integer, not double
        'date_added': '2026-03-01T00:00:00Z',
      };

      final track = Track.fromJson(json);
      expect(track.bpm, 128.0);
    });
  });

  group('Track formatting', () {
    test('durationFormatted returns -- for null', () {
      final track = Track.fromJson({
        'id': 't1',
        'title': 'T',
        'artist': 'A',
        'source': 'spotify',
        'date_added': '2026-03-01T00:00:00Z',
      });
      expect(track.durationFormatted, '--');
    });

    test('durationFormatted formats correctly', () {
      final track = Track.fromJson({
        'id': 't1',
        'title': 'T',
        'artist': 'A',
        'source': 'spotify',
        'duration_ms': 225000,
        'date_added': '2026-03-01T00:00:00Z',
      });
      expect(track.durationFormatted, '3:45');
    });

    test('bpmFormatted rounds to integer', () {
      final track = Track.fromJson({
        'id': 't1',
        'title': 'T',
        'artist': 'A',
        'source': 'spotify',
        'bpm': 128.7,
        'date_added': '2026-03-01T00:00:00Z',
      });
      expect(track.bpmFormatted, '129');
    });

    test('bpmFormatted returns -- for null', () {
      final track = Track.fromJson({
        'id': 't1',
        'title': 'T',
        'artist': 'A',
        'source': 'spotify',
        'date_added': '2026-03-01T00:00:00Z',
      });
      expect(track.bpmFormatted, '--');
    });

    test('keyFormatted returns key or -- for null', () {
      final withKey = Track.fromJson({
        'id': 't1',
        'title': 'T',
        'artist': 'A',
        'source': 'spotify',
        'key': 'Cm',
        'date_added': '2026-03-01T00:00:00Z',
      });
      expect(withKey.keyFormatted, 'Cm');

      final withoutKey = Track.fromJson({
        'id': 't2',
        'title': 'T',
        'artist': 'A',
        'source': 'spotify',
        'date_added': '2026-03-01T00:00:00Z',
      });
      expect(withoutKey.keyFormatted, '--');
    });
  });
}
