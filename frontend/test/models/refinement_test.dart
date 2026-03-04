import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/models/refinement.dart';
import 'package:ethnomusicology_frontend/models/setlist.dart';
import 'package:ethnomusicology_frontend/models/setlist_track.dart';

void main() {
  group('RefinementResponse', () {
    test('fromJson parses correctly', () {
      final json = {
        'version_number': 2,
        'tracks': [
          {
            'position': 1,
            'title': 'Track One',
            'artist': 'Artist A',
            'bpm': 128.0,
            'key': 'Am',
            'camelot': '8A',
            'energy': 7,
            'original_position': 1,
            'source': 'catalog',
            'track_id': 'trk-1',
          },
        ],
        'explanation': 'Made it more energetic',
        'change_warning': 'Removed 2 tracks',
      };

      final response = RefinementResponse.fromJson(json);

      expect(response.versionNumber, 2);
      expect(response.tracks, hasLength(1));
      expect(response.tracks[0].title, 'Track One');
      expect(response.tracks[0].bpm, 128.0);
      expect(response.explanation, 'Made it more energetic');
      expect(response.changeWarning, 'Removed 2 tracks');
    });

    test('fromJson handles null changeWarning', () {
      final json = {
        'version_number': 1,
        'tracks': [],
        'explanation': 'Initial version',
      };

      final response = RefinementResponse.fromJson(json);

      expect(response.changeWarning, isNull);
    });

    test('fromJson parses tracks with extra fields silently', () {
      final json = {
        'version_number': 1,
        'tracks': [
          {
            'id': 'ignored-id',
            'version_id': 'ignored-vid',
            'acquisition_info': 'ignored',
            'position': 1,
            'title': 'Test',
            'artist': 'Artist',
            'original_position': 1,
            'source': 'suggestion',
          },
        ],
        'explanation': 'Test',
      };

      final response = RefinementResponse.fromJson(json);
      expect(response.tracks[0].title, 'Test');
      expect(response.tracks[0].source, 'suggestion');
    });
  });

  group('SetlistVersion', () {
    test('fromJson parses all fields', () {
      final json = {
        'id': 'v-1',
        'setlist_id': 'set-1',
        'version_number': 3,
        'parent_version_id': 'v-0',
        'action': 'refine',
        'action_summary': 'More energy',
        'created_at': '2026-03-04T12:00:00Z',
      };

      final version = SetlistVersion.fromJson(json);

      expect(version.id, 'v-1');
      expect(version.setlistId, 'set-1');
      expect(version.versionNumber, 3);
      expect(version.parentVersionId, 'v-0');
      expect(version.action, 'refine');
      expect(version.actionSummary, 'More energy');
      expect(version.createdAt, '2026-03-04T12:00:00Z');
    });

    test('fromJson handles null optional fields', () {
      final json = {
        'id': 'v-1',
        'setlist_id': 'set-1',
        'version_number': 0,
      };

      final version = SetlistVersion.fromJson(json);

      expect(version.parentVersionId, isNull);
      expect(version.action, isNull);
      expect(version.actionSummary, isNull);
      expect(version.createdAt, isNull);
    });
  });

  group('ConversationMessage', () {
    test('fromJson parses user message', () {
      final json = {
        'id': 'msg-1',
        'setlist_id': 'set-1',
        'version_id': 'v-2',
        'role': 'user',
        'content': 'Make it more energetic',
        'created_at': '2026-03-04T12:00:00Z',
      };

      final msg = ConversationMessage.fromJson(json);

      expect(msg.id, 'msg-1');
      expect(msg.role, 'user');
      expect(msg.content, 'Make it more energetic');
      expect(msg.versionId, 'v-2');
    });

    test('fromJson handles null versionId', () {
      final json = {
        'id': 'msg-1',
        'setlist_id': 'set-1',
        'role': 'assistant',
        'content': 'Done!',
      };

      final msg = ConversationMessage.fromJson(json);

      expect(msg.versionId, isNull);
      expect(msg.createdAt, isNull);
    });
  });

  group('HistoryResponse', () {
    test('fromJson parses versions and conversations', () {
      final json = {
        'versions': [
          {
            'id': 'v-0',
            'setlist_id': 'set-1',
            'version_number': 0,
          },
          {
            'id': 'v-1',
            'setlist_id': 'set-1',
            'version_number': 1,
            'parent_version_id': 'v-0',
            'action': 'refine',
            'action_summary': 'More energy',
          },
        ],
        'conversations': [
          {
            'id': 'msg-1',
            'setlist_id': 'set-1',
            'role': 'user',
            'content': 'More energy please',
          },
          {
            'id': 'msg-2',
            'setlist_id': 'set-1',
            'role': 'assistant',
            'content': 'Added high-energy tracks',
          },
        ],
      };

      final history = HistoryResponse.fromJson(json);

      expect(history.versions, hasLength(2));
      expect(history.conversations, hasLength(2));
      expect(history.versions[1].actionSummary, 'More energy');
      expect(history.conversations[0].role, 'user');
    });

    test('fromJson handles empty lists', () {
      final json = {
        'versions': [],
        'conversations': [],
      };

      final history = HistoryResponse.fromJson(json);

      expect(history.versions, isEmpty);
      expect(history.conversations, isEmpty);
    });
  });

  group('Setlist.copyWith', () {
    final setlist = Setlist(
      id: 'set-1',
      prompt: 'test prompt',
      model: 'claude-sonnet',
      tracks: const [
        SetlistTrack(
          position: 1,
          title: 'Original',
          artist: 'Artist',
          originalPosition: 1,
          source: 'catalog',
        ),
      ],
      notes: 'some notes',
      harmonicFlowScore: 85.0,
      catalogPercentage: 90.0,
    );

    test('replaces tracks', () {
      final newTracks = [
        const SetlistTrack(
          position: 1,
          title: 'New Track',
          artist: 'New Artist',
          originalPosition: 1,
          source: 'suggestion',
        ),
      ];

      final updated = setlist.copyWith(tracks: newTracks);

      expect(updated.tracks[0].title, 'New Track');
      expect(updated.id, 'set-1');
      expect(updated.prompt, 'test prompt');
      expect(updated.notes, 'some notes');
    });

    test('preserves all fields when no args given', () {
      final copy = setlist.copyWith();

      expect(copy.id, setlist.id);
      expect(copy.prompt, setlist.prompt);
      expect(copy.model, setlist.model);
      expect(copy.tracks, setlist.tracks);
      expect(copy.notes, setlist.notes);
      expect(copy.harmonicFlowScore, setlist.harmonicFlowScore);
      expect(copy.catalogPercentage, setlist.catalogPercentage);
    });

    test('can set nullable fields to null', () {
      final updated = setlist.copyWith(
        notes: () => null,
        harmonicFlowScore: () => null,
      );

      expect(updated.notes, isNull);
      expect(updated.harmonicFlowScore, isNull);
    });
  });
}
