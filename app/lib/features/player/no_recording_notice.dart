import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import '../../shared_ui/state_notice.dart';
import 'player_texts.dart';

/// The empty state of the player: no recording yet, with the way to get one.
class NoRecordingNotice extends StatelessWidget {
  const NoRecordingNotice({required this.onChoose, super.key});

  final VoidCallback onChoose;

  @override
  Widget build(BuildContext context) => StateNotice(
    title: PlayerTexts.nothingSelected,
    message: PlayerTexts.nothingSelectedHint,
    icon: Icons.library_music_outlined,
    actionLabel: PlayerTexts.choose,
    actionIcon: Icons.folder_open,
    actionKind: ActionKind.primary,
    onAction: onChoose,
  );
}
