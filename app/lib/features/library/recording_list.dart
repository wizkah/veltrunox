import 'package:material_ui/material_ui.dart';

import 'library_texts.dart';
import 'recording_summary.dart';
import 'recording_tile.dart';

class RecordingList extends StatelessWidget {
  const RecordingList({
    required this.recordings,
    required this.selectedId,
    required this.onSelect,
    required this.onReanalyse,
    super.key,
  });

  final List<RecordingSummary> recordings;
  final int? selectedId;
  final ValueChanged<int> onSelect;
  final ValueChanged<int> onReanalyse;

  @override
  Widget build(BuildContext context) {
    if (recordings.isEmpty) {
      return const Center(child: Text(LibraryTexts.empty));
    }
    return ListView.builder(
      itemCount: recordings.length,
      itemBuilder: (context, index) {
        final recording = recordings[index];
        return RecordingTile(
          recording: recording,
          selected: recording.id == selectedId,
          onTap: () => onSelect(recording.id),
          onReanalyse: () => onReanalyse(recording.id),
        );
      },
    );
  }
}
