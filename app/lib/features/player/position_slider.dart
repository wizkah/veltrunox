import 'dart:async';

import 'package:material_ui/material_ui.dart';

import 'labeled_slider.dart';
import 'player_texts.dart';
import 'position_display.dart';

/// Moves through the recording. Moving the handle reports the position along
/// the way (scrubbing, a short piece is heard). Holding it still lets the
/// recording play on from there, as letting go would; letting go reports the
/// final position. Otherwise the slider follows the playback position.
class PositionSlider extends StatefulWidget {
  const PositionSlider({
    required this.positionMs,
    required this.lengthMs,
    required this.onScrub,
    required this.onScrubEnd,
    this.holdDelay = const Duration(milliseconds: 130),
    super.key,
  });

  final int positionMs;
  final int lengthMs;
  final ValueChanged<int> onScrub;
  final ValueChanged<int> onScrubEnd;

  /// How long the handle must rest before the recording plays on.
  final Duration holdDelay;

  @override
  State<PositionSlider> createState() => _PositionSliderState();
}

class _PositionSliderState extends State<PositionSlider> {
  Timer? _hold;

  /// The recording already plays on from the held handle; the release has
  /// nothing left to report (it would move the playback back).
  bool _playingOn = false;

  @override
  void dispose() {
    _hold?.cancel();
    super.dispose();
  }

  void _onMove(int position) {
    _hold?.cancel();
    _playingOn = false;
    widget.onScrub(position);
    _hold = Timer(widget.holdDelay, () {
      _playingOn = true;
      widget.onScrubEnd(position);
    });
  }

  void _onRelease(int position) {
    _hold?.cancel();
    if (!_playingOn) widget.onScrubEnd(position);
    _playingOn = false;
  }

  @override
  Widget build(BuildContext context) => LabeledSlider(
    label: PlayerTexts.position,
    value: widget.positionMs.toDouble(),
    min: 0,
    max: widget.lengthMs.toDouble(),
    format: (value) => formatPosition(value.round()),
    onChanging: (value) => _onMove(value.round()),
    onChanged: (value) => _onRelease(value.round()),
  );
}
