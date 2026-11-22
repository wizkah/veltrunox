import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'library_texts.dart';
import 'recording_summary.dart';
import 'recording_tile.dart';

/// The recordings as a large list. An empty list says why it is empty.
class RecordingList extends StatelessWidget {
  const RecordingList({
    required this.recordings,
    required this.detailId,
    required this.loadedId,
    required this.narrowed,
    required this.onShowDetails,
    required this.onPlay,
    super.key,
  });

  final List<RecordingSummary> recordings;
  final int? detailId;
  final int? loadedId;

  /// A search or filter is active, so an empty list is no empty library.
  final bool narrowed;
  final ValueChanged<int> onShowDetails;
  final ValueChanged<int> onPlay;

  @override
  Widget build(BuildContext context) {
    if (recordings.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(AppSpace.s6),
          child: Text(narrowed ? LibraryTexts.noMatch : LibraryTexts.empty),
        ),
      );
    }
    return ListView.builder(
      itemCount: recordings.length,
      itemBuilder: (context, index) {
        final recording = recordings[index];
        return RecordingTile(
          recording: recording,
          selected: recording.id == detailId,
          loaded: recording.id == loadedId,
          onTap: () => onShowDetails(recording.id),
          onPlay: () => onPlay(recording.id),
        );
      },
    );
  }
}
