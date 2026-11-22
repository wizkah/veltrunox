import 'package:material_ui/material_ui.dart';

import '../../shared_ui/app_icon_button.dart';
import 'player_texts.dart';

/// Sets a setting back to what the analysis found; unavailable when it already is.
class ResetButton extends StatelessWidget {
  const ResetButton({required this.onPressed, super.key});

  /// `null` when there is nothing to reset.
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) => AppIconButton(
    tooltip: PlayerTexts.resetToAnalysis,
    onPressed: onPressed,
    icon: Icons.restart_alt,
  );
}
