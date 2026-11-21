import 'package:material_ui/material_ui.dart';

import 'recording_summary.dart';

class RecordingTile extends StatelessWidget {
  const RecordingTile({
    required this.recording,
    required this.selected,
    required this.onTap,
    super.key,
  });

  final RecordingSummary recording;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => ListTile(
    leading: const Icon(Icons.music_note),
    title: Text(recording.title),
    selected: selected,
    onTap: onTap,
  );
}
