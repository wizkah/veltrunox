import 'package:material_ui/material_ui.dart';

import 'library_texts.dart';
import 'recording_summary.dart';
import 'recording_tile.dart';

class RecordingList extends StatelessWidget {
  const RecordingList({required this.recordings, super.key});

  final List<RecordingSummary> recordings;

  @override
  Widget build(BuildContext context) {
    if (recordings.isEmpty) {
      return const Center(child: Text(LibraryTexts.empty));
    }
    return ListView.builder(
      itemCount: recordings.length,
      itemBuilder: (context, index) =>
          RecordingTile(recording: recordings[index]),
    );
  }
}
