import 'package:material_ui/material_ui.dart';

import 'recording_summary.dart';

class RecordingTile extends StatelessWidget {
  const RecordingTile({required this.recording, super.key});

  final RecordingSummary recording;

  @override
  Widget build(BuildContext context) => ListTile(
    leading: const Icon(Icons.music_note),
    title: Text(recording.title),
  );
}
