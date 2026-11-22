import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'player_texts.dart';

/// The empty state of the player: no recording yet, with the way to get one.
class NoRecordingNotice extends StatelessWidget {
  const NoRecordingNotice({required this.onChoose, super.key});

  final VoidCallback onChoose;

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.all(AppSpace.s6),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(
            Icons.library_music_outlined,
            size: AppSpace.s8 * 2,
            color: AppColors.accentSoft,
          ),
          const SizedBox(height: AppSpace.s3),
          Text(
            PlayerTexts.nothingSelected,
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: AppSpace.s2),
          const Text(
            PlayerTexts.nothingSelectedHint,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: AppSpace.s4),
          ActionButton(label: PlayerTexts.choose, onPressed: onChoose),
        ],
      ),
    ),
  );
}
