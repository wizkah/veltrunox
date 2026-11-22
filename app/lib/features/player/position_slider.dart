import 'package:material_ui/material_ui.dart';

import 'labeled_slider.dart';
import 'player_texts.dart';
import 'position_display.dart';
import 'scrub_hold.dart';

/// Identifies the position slider.
const positionSliderKey = ValueKey('position-slider');

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

  /// `null` while the length is not known (the recording is not scanned yet):
  /// the slider is shown but cannot be moved.
  final int? lengthMs;
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
  late final ScrubHold _scrub = ScrubHold(
    onScrub: (position) => widget.onScrub(position),
    onScrubEnd: (position) => widget.onScrubEnd(position),
    onHold: (position) => widget.onHold(position),
    onHoldEnd: () => widget.onHoldEnd(),
    holdDelay: widget.holdDelay,
  );

  @override
  void dispose() {
    _scrub.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final length = widget.lengthMs;
    final known = length != null && length > 0;
    return LabeledSlider(
      key: positionSliderKey,
      label: PlayerTexts.position,
      enabled: known,
      value: known ? widget.positionMs.toDouble() : 0,
      min: 0,
      max: known ? length.toDouble() : 1,
      format: (value) => formatPosition(value.round()),
      onChanging: (value) => _scrub.moved(value.round()),
      onChanged: (value) => _scrub.released(value.round()),
    );
  }
}
