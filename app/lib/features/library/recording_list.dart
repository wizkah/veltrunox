import 'package:material_ui/material_ui.dart';

import '../../shared_ui/state_notice.dart';
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
      return StateNotice(
        message: narrowed ? LibraryTexts.noMatch : LibraryTexts.empty,
        icon: narrowed ? Icons.search_off : Icons.library_music_outlined,
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
