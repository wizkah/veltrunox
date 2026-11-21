import 'package:material_ui/material_ui.dart';

/// A slider with a label and value text. While dragging it shows the dragged
/// value; the change is reported once when the drag ends.
class LabeledSlider extends StatefulWidget {
  const LabeledSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.format,
    required this.onChanged,
    this.trailing,
    super.key,
  });

  final String label;
  final double value;
  final double min;
  final double max;
  final String Function(double) format;
  final ValueChanged<double> onChanged;

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
        SizedBox(width: 120, child: Text(widget.label)),
        Expanded(
          child: Slider(
            value: shown.clamp(widget.min, widget.max),
            min: widget.min,
            max: widget.max,
            onChanged: (value) => setState(() => _dragged = value),
            onChangeEnd: (value) {
              setState(() => _dragged = null);
              widget.onChanged(value);
            },
          ),
        ),
        SizedBox(width: 72, child: Text(widget.format(shown))),
        ?widget.trailing,
      ],
    );
  }
}
