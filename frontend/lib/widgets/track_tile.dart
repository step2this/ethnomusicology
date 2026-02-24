import 'package:flutter/material.dart';

import '../models/track.dart';

class TrackTile extends StatelessWidget {
  final Track track;
  final VoidCallback? onTap;

  const TrackTile({
    super.key,
    required this.track,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final minutes = track.duration.inMinutes;
    final seconds = track.duration.inSeconds.remainder(60);
    final durationText = '$minutes:${seconds.toString().padLeft(2, '0')}';

    return ListTile(
      title: Text(track.title),
      subtitle: Text('${track.artist} • ${track.album}'),
      trailing: Text(durationText),
      onTap: onTap,
    );
  }
}
