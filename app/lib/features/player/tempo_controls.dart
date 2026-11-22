import 'package:material_ui/material_ui.dart';

import 'labeled_slider.dart';
import 'player_texts.dart';
import 'reset_button.dart';
import 'target_tempo_field.dart';
import 'throttled_value.dart';

/// The speed of the playback. It takes effect while the slider is dragged, and
/// with the recording's tempo known the wanted tempo follows the handle at once
/// and can be typed in.
class TempoControls extends StatefulWidget {
  const TempoControls({
    required this.speed,
    required this.onSpeed,
    required this.onTargetTempo,
    this.recordingBpm,
    super.key,
  });

  final double speed;

  /// Tempo of the recording from the analysis, if it has one.
  final double? recordingBpm;
  final ValueChanged<double> onSpeed;
  final Future<bool> Function(double bpm) onTargetTempo;

  @override
  State<TempoControls> createState() => _TempoControlsState();
}

class _TempoControlsState extends State<TempoControls> {
  late final ThrottledValue<double> _live = ThrottledValue(
    (speed) => widget.onSpeed(speed),
  );

  /// The speed under the handle while it is dragged.
  double? _dragged;

  @override
  void dispose() {
    _live.dispose();
    super.dispose();
  }

  void _onChanging(double speed) {
    setState(() => _dragged = speed);
    _live.add(speed);
  }

  void _onChanged(double speed) {
    _live.cancel();
    setState(() => _dragged = null);
    widget.onSpeed(speed);
  }

  @override
  Widget build(BuildContext context) {
    final bpm = widget.recordingBpm;
    final speed = _dragged ?? widget.speed;
    return Column(
      children: [
        LabeledSlider(
          label: PlayerTexts.speed,
          value: widget.speed,
          min: 0.5,
          max: 2,
          format: PlayerTexts.speedValue,
          onChanging: _onChanging,
          onChanged: _onChanged,
          trailing: bpm == null
              ? null
              : ResetButton(
                  onPressed: widget.speed == 1 ? null : () => widget.onSpeed(1),
                ),
        ),
        if (bpm != null)
          TargetTempoField(
            currentBpm: bpm * speed,
            onSubmit: widget.onTargetTempo,
          ),
      ],
    );
  }
}
