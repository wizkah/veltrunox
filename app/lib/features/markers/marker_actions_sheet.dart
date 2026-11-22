import 'package:material_ui/material_ui.dart';

import 'marker.dart';
import 'marker_texts.dart';

enum MarkerAction { edit, delete }

/// Asks what to do with a marker; the answer is `null` when dismissed. Reached
/// by a long press on a marker in the waveform.
Future<MarkerAction?> showMarkerActions(BuildContext context, Marker marker) =>
    showModalBottomSheet<MarkerAction>(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(title: Text(marker.title)),
            ListTile(
              leading: const Icon(Icons.edit_outlined),
              title: const Text(MarkerTexts.edit),
              onTap: () => Navigator.of(context).pop(MarkerAction.edit),
            ),
            ListTile(
              leading: const Icon(Icons.delete_outline),
              title: const Text(MarkerTexts.delete),
              onTap: () => Navigator.of(context).pop(MarkerAction.delete),
            ),
          ],
        ),
      ),
    );
