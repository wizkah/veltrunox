import 'package:material_ui/material_ui.dart';

import 'key_stepper.dart';
import 'labeled_slider.dart';
import 'player_analysis.dart';
import 'player_state.dart';
import 'player_texts.dart';
import 'reset_button.dart';
import 'target_tempo_field.dart';

/// Speed, tuning and key of the playback. The server enforces the limits;
/// the controls only offer the usual range. With an [analysis] they also show
/// the tempo, take a wanted tempo and set each setting back to what was found.
class SettingsControls extends StatelessWidget {
  const SettingsControls({
    required this.state,
    required this.onSpeed,
    required this.onTuning,
    required this.onKey,
    required this.onTargetTempo,
    this.analysis,
    super.key,
  });

  final PlayerState state;
  final PlayerAnalysis? analysis;
  final ValueChanged<double> onSpeed;
  final ValueChanged<double> onTuning;
  final ValueChanged<int> onKey;
  final Future<bool> Function(double bpm) onTargetTempo;

  @override
  Widget build(BuildContext context) {
    final found = analysis;
    return Column(
      children: [
        LabeledSlider(
          label: PlayerTexts.speed,
          value: state.speed,
          min: 0.5,
          max: 2,
          format: PlayerTexts.speedValue,
          onChanged: onSpeed,
          trailing: found == null
              ? null
              : ResetButton(
                  onPressed: state.speed == 1 ? null : () => onSpeed(1),
                ),
        ),
        if (found != null)
          TargetTempoField(
            currentBpm: found.tempoBpm * state.speed,
            onSubmit: onTargetTempo,
          ),
        LabeledSlider(
          label: PlayerTexts.tuning,
          value: state.tuningHz,
          min: 350,
          max: 500,
          format: PlayerTexts.tuningValue,
          onChanged: onTuning,
          trailing: found == null
              ? null
              : ResetButton(
                  onPressed: state.tuningHz == found.tuningHz
                      ? null
                      : () => onTuning(found.tuningHz),
                ),
        ),
        KeyStepper(
          semitones: state.keyShiftSemitones,
          onChanged: onKey,
          trailing: found == null
              ? null
              : ResetButton(
                  onPressed: state.keyShiftSemitones == 0
                      ? null
                      : () => onKey(0),
                ),
        ),
      ],
    );
  }
}
