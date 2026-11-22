import 'package:material_ui/material_ui.dart';

import '../../design/icons/metronome_icon.dart';
import '../../design/icons/tuning_fork_icon.dart';
import '../../design/tokens/app_geometry.dart';
import '../library/analysis_format.dart';
import 'key_dropdown.dart';
import 'player_analysis.dart';
import 'player_state.dart';
import 'player_texts.dart';
import 'reset_button.dart';
import 'settings_panel.dart';
import 'tempo_controls.dart';
import 'tuning_field.dart';

/// Tempo and pitch of the playback, each in a panel. The server enforces the
/// limits; the controls only offer the usual range. With an [analysis] they
/// also show the tempo, take a wanted tempo and set each setting back to what
/// was found.
class SettingsControls extends StatelessWidget {
  const SettingsControls({
    required this.state,
    required this.onSpeed,
    required this.onTuning,
    required this.onKey,
    required this.onTargetTempo,
    this.analysis,
    this.sideBySide = false,
    super.key,
  });

  final PlayerState state;
  final PlayerAnalysis? analysis;
  final ValueChanged<double> onSpeed;
  final Future<bool> Function(double hertz) onTuning;
  final ValueChanged<int> onKey;
  final Future<bool> Function(double bpm) onTargetTempo;

  /// The panels stand next to each other instead of one above the other.
  final bool sideBySide;

  @override
  Widget build(BuildContext context) {
    final panels = [_tempoPanel(), _pitchPanel()];
    if (!sideBySide) {
      return Column(
        children: [
          for (final (index, panel) in panels.indexed) ...[
            if (index > 0) const SizedBox(height: AppSpace.s3),
            panel,
          ],
        ],
      );
    }
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final (index, panel) in panels.indexed) ...[
          if (index > 0) const SizedBox(width: AppSpace.s4),
          Expanded(child: panel),
        ],
      ],
    );
  }

  Widget _tempoPanel() {
    final found = analysis;
    return SettingsPanel(
      icon: const MetronomeIcon(),
      title: PlayerTexts.tempo,
      note: found == null
          ? null
          : PlayerTexts.original(PlayerTexts.tempoValue(found.tempoBpm)),
      child: TempoControls(
        speed: state.speed,
        recordingBpm: found?.tempoBpm,
        onSpeed: onSpeed,
        onTargetTempo: onTargetTempo,
      ),
    );
  }

  Widget _pitchPanel() {
    final found = analysis;
    return SettingsPanel(
      icon: const TuningForkIcon(),
      title: PlayerTexts.pitch,
      note: found == null ? null : _pitchNote(found),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          TuningField(
            currentHz: state.tuningHz,
            onSubmit: onTuning,
            trailing: ResetButton(
              onPressed: found == null || state.tuningHz == found.tuningHz
                  ? null
                  : () => onTuning(found.tuningHz),
            ),
          ),
          KeyDropdown(
            semitones: state.keyShiftSemitones,
            onChanged: onKey,
            originalTonic: found?.keyTonic,
            originalMinor: found?.keyMinor ?? false,
            trailing: ResetButton(
              onPressed: state.keyShiftSemitones == 0 ? null : () => onKey(0),
            ),
          ),
        ],
      ),
    );
  }

  /// "Original: 443 Hz · E-Dur".
  String _pitchNote(PlayerAnalysis found) {
    final hertz = PlayerTexts.tuningValue(found.tuningHz);
    final tonic = found.keyTonic;
    final key = tonic == null
        ? ''
        : ' · ${keyName(tonic, minor: found.keyMinor)}';
    return PlayerTexts.original('$hertz$key');
  }
}
