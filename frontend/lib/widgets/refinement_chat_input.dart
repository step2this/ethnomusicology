import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/refinement_provider.dart';

class RefinementChatInput extends ConsumerStatefulWidget {
  final String setlistId;
  final bool isRefining;

  const RefinementChatInput({
    super.key,
    required this.setlistId,
    required this.isRefining,
  });

  @override
  ConsumerState<RefinementChatInput> createState() => _RefinementChatInputState();
}

class _RefinementChatInputState extends ConsumerState<RefinementChatInput> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _send(String message) {
    if (message.trim().isEmpty || widget.isRefining) return;
    ref.read(refinementProvider.notifier).refineSetlist(widget.setlistId, message.trim());
    _controller.clear();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Quick command chips
        SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          child: Row(
            children: [
              _buildChip('!shuffle', Icons.shuffle),
              const SizedBox(width: 4),
              _buildChip('!sort-by-bpm', Icons.speed),
              const SizedBox(width: 4),
              _buildChip('!reverse', Icons.swap_vert),
              const SizedBox(width: 4),
              _buildChip('!undo', Icons.undo),
            ],
          ),
        ),
        // Text input
        Padding(
          padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _controller,
                  enabled: !widget.isRefining,
                  decoration: InputDecoration(
                    hintText: 'Refine your setlist...',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(24),
                    ),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                    isDense: true,
                  ),
                  onSubmitted: _send,
                  textInputAction: TextInputAction.send,
                ),
              ),
              const SizedBox(width: 8),
              IconButton.filled(
                onPressed: widget.isRefining ? null : () => _send(_controller.text),
                icon: widget.isRefining
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.send),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildChip(String label, IconData icon) {
    return ActionChip(
      avatar: Icon(icon, size: 16),
      label: Text(label, style: const TextStyle(fontSize: 12)),
      onPressed: widget.isRefining ? null : () => _send(label),
      visualDensity: VisualDensity.compact,
    );
  }
}
