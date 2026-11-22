import 'package:material_ui/material_ui.dart';

/// A slider with value text; the label is its accessible name, not shown. While dragging it shows the dragged
/// value; the change is reported once when the drag ends (and, with
/// `onChanging`, along the way).
class LabeledSlider extends StatefulWidget {
  const LabeledSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.format,
    required this.onChanged,
    this.onChanging,
    this.trailing,
    this.enabled = true,
    super.key,
  });

  final String label;
  final double value;
  final double min;
  final double max;
  final String Function(double) format;
  final ValueChanged<double> onChanged;

  /// Reports every value while the slider is dragged; for settings that should
  /// take effect at once.
  final ValueChanged<double>? onChanging;

  /// A disabled slider shows its value but cannot be moved.
  final bool enabled;

  /// Shown after the value, for example a reset button.
  final Widget? trailing;

  @override
  State<LabeledSlider> createState() => _LabeledSliderState();
}

class _LabeledSliderState extends State<LabeledSlider> {
  double? _dragged;

  @override
  Widget build(BuildContext context) {
    final shown = _dragged ?? widget.value;
    return Row(
      children: [
        Expanded(
          child: Semantics(
            label: widget.label,
            child: Slider(
              value: shown.clamp(widget.min, widget.max),
              min: widget.min,
              max: widget.max,
              onChanged: widget.enabled
                  ? (value) {
                      setState(() => _dragged = value);
                      widget.onChanging?.call(value);
                    }
                  : null,
              onChangeEnd: (value) {
                setState(() => _dragged = null);
                widget.onChanged(value);
              },
            ),
          ),
        ),
        SizedBox(width: 72, child: Text(widget.format(shown))),
        ?widget.trailing,
      ],
    );
  }
}
