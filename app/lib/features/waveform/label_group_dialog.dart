import 'package:material_ui/material_ui.dart';

import '../../shared_ui/clock_format.dart';
import '../markers/marker.dart';
import 'label_clusters.dart';
import 'waveform_texts.dart';

/// Lists the labels of a group; choosing one is the second step of playing it.
Future<LabelMarker?> showLabelGroupDialog(
  BuildContext context,
  LabelCluster cluster,
) => showDialog<LabelMarker>(
  context: context,
  builder: (context) => SimpleDialog(
    title: Text(WaveformTexts.groupTitle),
    children: [
      for (final label in cluster.labels)
        SimpleDialogOption(
          onPressed: () => Navigator.of(context).pop(label),
          child: ListTile(
            leading: const Icon(Icons.play_arrow),
            title: Text(
              label.title.isEmpty
                  ? WaveformTexts.fallbackLabel(label.id)
                  : label.title,
            ),
            subtitle: Text(formatClock(label.positionMs)),
          ),
        ),
    ],
  ),
);
