import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'panel_surface.dart';
import 'player_texts.dart';
import 'throttled_value.dart';

/// Identifies the volume slider.
const volumeSliderKey = ValueKey('volume-slider');

/// The output volume of the server. Everyone sees the change at once, so the
/// slider reports while it is dragged, not only when it is let go. It lies
/// flat in a row or, with [vertical], stands with the maximum at the top and
/// takes the height it is given.
class VolumeControl extends StatefulWidget {
  const VolumeControl({
    required this.percent,
    required this.onChanged,
    this.vertical = false,
    super.key,
  });

  final int percent;
  final ValueChanged<int> onChanged;
  final bool vertical;

  @override
  State<VolumeControl> createState() => _VolumeControlState();
}

class _VolumeControlState extends State<VolumeControl> {
  static const _sliderLength = 160.0;

  late final ThrottledValue<int> _live = ThrottledValue(
    (percent) => widget.onChanged(percent),
  );

  /// The volume under the handle while it is dragged.
  double? _dragged;

  @override
  void dispose() {
    _live.dispose();
    super.dispose();
  }

  Widget _slider(double shown) =>
      KeyedSubtree(key: volumeSliderKey, child: _volumeSlider(shown));

  Widget _volumeSlider(double shown) => Slider(
    value: shown.clamp(0, 100),
    min: 0,
    max: 100,
    onChanged: (value) {
      setState(() => _dragged = value);
      _live.add(value.round());
    },
    onChangeEnd: (value) {
      setState(() => _dragged = null);
      _live.cancel();
      widget.onChanged(value.round());
    },
  );

  @override
  Widget build(BuildContext context) {
    final shown = _dragged ?? widget.percent.toDouble();
    final value = Text(PlayerTexts.volumeValue(shown));
    final icon = Icon(
      shown == 0 ? Icons.volume_off : Icons.volume_up,
      color: Theme.of(context).colorScheme.tertiary,
    );
    if (widget.vertical) {
      return PanelSurface(
        child: Column(
          children: [
            icon,
            Text(PlayerTexts.volume),
            const SizedBox(height: AppSpace.s2),
            Expanded(
              child: ConstrainedBox(
                constraints: const BoxConstraints(minHeight: _sliderLength),
                child: RotatedBox(quarterTurns: 3, child: _slider(shown)),
              ),
            ),
            value,
          ],
        ),
      );
    }
    return PanelSurface(
      child: Row(
        children: [
          icon,
          const SizedBox(width: AppSpace.s2),
          Text(PlayerTexts.volume),
          Expanded(child: _slider(shown)),
          SizedBox(width: 64, child: Text(PlayerTexts.volumeValue(shown))),
        ],
      ),
    );
  }
}
