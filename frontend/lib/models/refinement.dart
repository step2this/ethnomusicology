import 'setlist_track.dart';

class RefinementResponse {
  final int versionNumber;
  final List<SetlistTrack> tracks;
  final String explanation;
  final String? changeWarning;

  const RefinementResponse({
    required this.versionNumber,
    required this.tracks,
    required this.explanation,
    this.changeWarning,
  });

  factory RefinementResponse.fromJson(Map<String, dynamic> json) {
    return RefinementResponse(
      versionNumber: json['version_number'] as int,
      tracks: (json['tracks'] as List<dynamic>)
          .map((t) => SetlistTrack.fromJson(t as Map<String, dynamic>))
          .toList(),
      explanation: json['explanation'] as String,
      changeWarning: json['change_warning'] as String?,
    );
  }
}

class SetlistVersion {
  final String id;
  final String setlistId;
  final int versionNumber;
  final String? parentVersionId;
  final String? action;
  final String? actionSummary;
  final String? createdAt;

  const SetlistVersion({
    required this.id,
    required this.setlistId,
    required this.versionNumber,
    this.parentVersionId,
    this.action,
    this.actionSummary,
    this.createdAt,
  });

  factory SetlistVersion.fromJson(Map<String, dynamic> json) {
    return SetlistVersion(
      id: json['id'] as String,
      setlistId: json['setlist_id'] as String,
      versionNumber: json['version_number'] as int,
      parentVersionId: json['parent_version_id'] as String?,
      action: json['action'] as String?,
      actionSummary: json['action_summary'] as String?,
      createdAt: json['created_at'] as String?,
    );
  }
}

class ConversationMessage {
  final String id;
  final String setlistId;
  final String? versionId;
  final String role; // "user" or "assistant"
  final String content;
  final String? createdAt;

  const ConversationMessage({
    required this.id,
    required this.setlistId,
    this.versionId,
    required this.role,
    required this.content,
    this.createdAt,
  });

  factory ConversationMessage.fromJson(Map<String, dynamic> json) {
    return ConversationMessage(
      id: json['id'] as String,
      setlistId: json['setlist_id'] as String,
      versionId: json['version_id'] as String?,
      role: json['role'] as String,
      content: json['content'] as String,
      createdAt: json['created_at'] as String?,
    );
  }
}

class HistoryResponse {
  final List<SetlistVersion> versions;
  final List<ConversationMessage> conversations;

  const HistoryResponse({
    required this.versions,
    required this.conversations,
  });

  factory HistoryResponse.fromJson(Map<String, dynamic> json) {
    return HistoryResponse(
      versions: (json['versions'] as List<dynamic>)
          .map((v) => SetlistVersion.fromJson(v as Map<String, dynamic>))
          .toList(),
      conversations: (json['conversations'] as List<dynamic>)
          .map((c) => ConversationMessage.fromJson(c as Map<String, dynamic>))
          .toList(),
    );
  }
}
