import 'package:material_ui/material_ui.dart';

import 'labeled_slider.dart';
import 'player_texts.dart';
import 'throttled_value.dart';

/// The output volume of the server. Everyone sees the change at once, so the
/// slider reports while it is dragged, not only when it is let go.
class VolumeControl extends StatefulWidget {
  const VolumeControl({
    required this.percent,
    required this.onChanged,
    super.key,
  });

  final int percent;
  final ValueChanged<int> onChanged;

  @override
  State<VolumeControl> createState() => _VolumeControlState();
}

class _VolumeControlState extends State<VolumeControl> {
  late final ThrottledValue<int> _live = ThrottledValue(
    (percent) => widget.onChanged(percent),
  );

  @override
  void dispose() {
    _live.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => LabeledSlider(
    label: PlayerTexts.volume,
    value: widget.percent.toDouble(),
    min: 0,
    max: 100,
    format: PlayerTexts.volumeValue,
    onChanging: (value) => _live.add(value.round()),
    onChanged: (value) {
      _live.cancel();
      widget.onChanged(value.round());
    },
  );
}
