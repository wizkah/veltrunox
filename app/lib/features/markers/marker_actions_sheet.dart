import 'package:material_ui/material_ui.dart';

import 'marker.dart';
import 'marker_texts.dart';

enum MarkerAction {
  edit,
  duplicate,
  split,
  delete;

  /// What can be done with this marker; only a region is duplicated or split.
  static List<MarkerAction> of(Marker marker) => switch (marker) {
    LabelMarker() => const [edit, delete],
    RegionMarker() => values,
  };

  String get label => switch (this) {
    edit => MarkerTexts.edit,
    duplicate => MarkerTexts.duplicate,
    split => MarkerTexts.splitAtPlayhead,
    delete => MarkerTexts.delete,
  };

  IconData get icon => switch (this) {
    edit => Icons.edit_outlined,
    duplicate => Icons.content_copy,
    split => Icons.content_cut,
    delete => Icons.delete_outline,
  };
}

/// Asks what to do with a marker; the answer is `null` when dismissed. Reached
/// by a long press on a marker in the waveform.
Future<MarkerAction?> showMarkerActions(BuildContext context, Marker marker) =>
    showModalBottomSheet<MarkerAction>(
      context: context,
      builder: (context) => SafeArea(
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListTile(title: Text(marker.title)),
              for (final action in MarkerAction.of(marker))
                ListTile(
                  leading: Icon(action.icon),
                  title: Text(action.label),
                  onTap: () => Navigator.of(context).pop(action),
                ),
            ],
          ),
        ),
      ),
    );
