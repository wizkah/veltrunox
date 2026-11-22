import 'package:material_ui/material_ui.dart';

import 'labeled_slider.dart';
import 'player_texts.dart';
import 'position_display.dart';

/// Moves through the recording. Dragging reports the position along the way
/// (scrubbing); letting go reports the final one. Otherwise it follows the
/// playback position.
class PositionSlider extends StatelessWidget {
  const PositionSlider({
    required this.positionMs,
    required this.lengthMs,
    required this.onScrub,
    required this.onScrubEnd,
    super.key,
  });

  final int positionMs;
  final int lengthMs;
  final ValueChanged<int> onScrub;
  final ValueChanged<int> onScrubEnd;

  @override
  Widget build(BuildContext context) => LabeledSlider(
    label: PlayerTexts.position,
    value: positionMs.toDouble(),
    min: 0,
    max: lengthMs.toDouble(),
    format: (value) => formatPosition(value.round()),
    onChanging: (value) => onScrub(value.round()),
    onChanged: (value) => onScrubEnd(value.round()),
  );
}
