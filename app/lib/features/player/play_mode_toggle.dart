import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import 'player_state.dart';
import 'player_texts.dart';

/// The three-state play mode next to the play button. A tap switches to the
/// next mode; the active one shows as icon, text and colour. It only matters
/// when a region is played.
class PlayModeToggle extends StatelessWidget {
  const PlayModeToggle({
    required this.mode,
    required this.onPressed,
    super.key,
  });

  final PlayMode mode;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) => Tooltip(
    message: PlayerTexts.playModeTooltip(_label),
    child: OutlinedButton.icon(
      onPressed: onPressed,
      style: OutlinedButton.styleFrom(foregroundColor: _color),
      icon: Icon(_icon),
      label: Text(_label, semanticsLabel: PlayerTexts.playModeTooltip(_label)),
    ),
  );

  IconData get _icon => switch (mode) {
    PlayMode.toEnd => Icons.last_page,
    PlayMode.toRegionEnd => Icons.vertical_align_center,
    PlayMode.loop => Icons.repeat,
  };

  String get _label => switch (mode) {
    PlayMode.toEnd => PlayerTexts.playModeToEnd,
    PlayMode.toRegionEnd => PlayerTexts.playModeToRegionEnd,
    PlayMode.loop => PlayerTexts.playModeLoop,
  };

  Color get _color => switch (mode) {
    PlayMode.toEnd => AppColors.textPrimary,
    PlayMode.toRegionEnd => AppColors.accentSoft,
    PlayMode.loop => AppColors.action,
  };
}
