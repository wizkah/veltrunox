import 'dart:async';

import 'package:material_ui/material_ui.dart';

import 'labeled_slider.dart';
import 'player_texts.dart';
import 'position_display.dart';

/// Moves through the recording. Moving the handle reports the position along
/// the way (scrubbing, a short piece is heard). Holding it still lets the
/// recording play on from there; letting go reports the final position, or
/// after a rest only that the handle is let go. Otherwise the slider follows
/// the playback position.
class PositionSlider extends StatefulWidget {
  const PositionSlider({
    required this.positionMs,
    required this.lengthMs,
    required this.onScrub,
    required this.onScrubEnd,
    required this.onHold,
    required this.onHoldEnd,
    this.holdDelay = const Duration(milliseconds: 130),
    super.key,
  });

  final int positionMs;
  final int lengthMs;
  final ValueChanged<int> onScrub;
  final ValueChanged<int> onScrubEnd;

  /// The handle rests at this position: the recording plays on from there.
  final ValueChanged<int> onHold;

  /// The handle is let go after it rested.
  final VoidCallback onHoldEnd;

  /// How long the handle must rest before the recording plays on.
  final Duration holdDelay;

  @override
  State<PositionSlider> createState() => _PositionSliderState();
}

class _PositionSliderState extends State<PositionSlider> {
  Timer? _hold;

  /// The recording already plays on from the held handle; the release must not
  /// move the playback back.
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
      widget.onHold(position);
    });
  }

  void _onRelease(int position) {
    _hold?.cancel();
    if (_playingOn) {
      widget.onHoldEnd();
    } else {
      widget.onScrubEnd(position);
    }
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
