import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';

import '../models/purchase_link.dart';
import '../providers/api_provider.dart';

class PurchaseLinkPanel extends ConsumerStatefulWidget {
  final String title;
  final String artist;

  const PurchaseLinkPanel({
    super.key,
    required this.title,
    required this.artist,
  });

  @override
  ConsumerState<PurchaseLinkPanel> createState() => _PurchaseLinkPanelState();
}

class _PurchaseLinkPanelState extends ConsumerState<PurchaseLinkPanel> {
  bool _expanded = false;
  bool _loading = false;
  List<PurchaseLink>? _links;
  String? _error;

  bool get _hasTrackInfo => widget.title.isNotEmpty && widget.artist.isNotEmpty;

  Future<void> _fetchLinks() async {
    if (!_hasTrackInfo) return;
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final links = await ref.read(apiClientProvider).getPurchaseLinks(
            title: widget.title,
            artist: widget.artist,
          );
      if (mounted) {
        setState(() {
          _links = links;
          _loading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = 'Failed to load purchase links';
          _loading = false;
        });
      }
    }
  }

  void _toggle() {
    setState(() {
      _expanded = !_expanded;
    });
    if (_expanded && _links == null && _error == null) {
      _fetchLinks();
    }
  }

  Future<void> _openLink(PurchaseLink link) async {
    final uri = Uri.parse(link.url);
    try {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    } catch (_) {
      if (!mounted) return;
      await Clipboard.setData(ClipboardData(text: link.url));
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Link copied: ${link.name}')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    if (!_hasTrackInfo) {
      return const SizedBox.shrink();
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: _toggle,
          borderRadius: BorderRadius.circular(4),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 2),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.shopping_bag_outlined,
                  size: 16,
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 4),
                Text(
                  'Buy',
                  style: theme.textTheme.labelSmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
                Icon(
                  _expanded ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ],
            ),
          ),
        ),
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeInOut,
          alignment: Alignment.topLeft,
          child: _expanded ? _buildContent(theme) : const SizedBox.shrink(),
        ),
      ],
    );
  }

  Widget _buildContent(ThemeData theme) {
    if (_loading) {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 8),
        child: SizedBox(
          width: 16,
          height: 16,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      );
    }

    if (_error != null) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Text(
          _error!,
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.error,
          ),
        ),
      );
    }

    if (_links != null && _links!.isEmpty) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Text(
          'No purchase links',
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
      );
    }

    if (_links == null) {
      return const SizedBox.shrink();
    }

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Wrap(
        spacing: 6,
        runSpacing: 4,
        children: _links!.map((link) => _buildChip(theme, link)).toList(),
      ),
    );
  }

  Widget _buildChip(ThemeData theme, PurchaseLink link) {
    return ActionChip(
      avatar: Text(link.icon, style: const TextStyle(fontSize: 14)),
      label: Text(link.name),
      labelStyle: theme.textTheme.labelSmall,
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      visualDensity: VisualDensity.compact,
      side: BorderSide(color: theme.colorScheme.outlineVariant),
      onPressed: () => _openLink(link),
    );
  }
}
