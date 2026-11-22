import 'package:material_ui/material_ui.dart';

import 'key_stepper.dart';
import 'player_analysis.dart';
import 'player_state.dart';
import 'reset_button.dart';
import 'tempo_controls.dart';
import 'tuning_field.dart';

/// Speed, tuning and key of the playback. The server enforces the limits;
/// the controls only offer the usual range and the tuning is typed in. With an [analysis] they also show
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
  final Future<bool> Function(double hertz) onTuning;
  final ValueChanged<int> onKey;
  final Future<bool> Function(double bpm) onTargetTempo;

  @override
  Widget build(BuildContext context) {
    final found = analysis;
    return Column(
      children: [
        TempoControls(
          speed: state.speed,
          recordingBpm: found?.tempoBpm,
          onSpeed: onSpeed,
          onTargetTempo: onTargetTempo,
        ),
        TuningField(
          currentHz: state.tuningHz,
          onSubmit: onTuning,
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
