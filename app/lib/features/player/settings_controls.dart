import 'package:material_ui/material_ui.dart';

import 'key_stepper.dart';
import 'labeled_slider.dart';
import 'player_state.dart';
import 'player_texts.dart';

/// Speed, tuning and key of the playback. The server enforces the limits;
/// the controls only offer the usual range.
class SettingsControls extends StatelessWidget {
  const SettingsControls({
    required this.state,
    required this.onSpeed,
    required this.onTuning,
    required this.onKey,
    super.key,
  });

  final PlayerState state;
  final ValueChanged<double> onSpeed;
  final ValueChanged<double> onTuning;
  final ValueChanged<int> onKey;

  @override
  Widget build(BuildContext context) => Column(
    children: [
      LabeledSlider(
        label: PlayerTexts.speed,
        value: state.speed,
        min: 0.5,
        max: 2,
        format: PlayerTexts.speedValue,
        onChanged: onSpeed,
      ),
      LabeledSlider(
        label: PlayerTexts.tuning,
        value: state.tuningHz,
        min: 350,
        max: 500,
        format: PlayerTexts.tuningValue,
        onChanged: onTuning,
      ),
      KeyStepper(semitones: state.keyShiftSemitones, onChanged: onKey),
    ],
  );
}
