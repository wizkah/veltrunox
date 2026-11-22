import 'package:material_ui/material_ui.dart';

import 'number_input.dart';

/// Shows a value and takes a wanted number for it. Confirming (Enter) hands the
/// number on; the field clears when it was taken and shows a message when not.
class ValueEntryField extends StatefulWidget {
  const ValueEntryField({
    required this.label,
    required this.currentText,
    required this.fieldLabel,
    required this.notANumber,
    required this.outOfRange,
    required this.onSubmit,
    this.trailing,
    super.key,
  });

  final String label;

  /// The value as it stands now, with its unit.
  final String currentText;
  final String fieldLabel;

  /// Messages for text that is no number and for a number that was not taken.
  final String notANumber;
  final String outOfRange;

  /// Reports whether the number could be set.
  final Future<bool> Function(double value) onSubmit;

  /// Shown after the field, for example a reset button.
  final Widget? trailing;

  @override
  State<ValueEntryField> createState() => _ValueEntryFieldState();
}

class _ValueEntryFieldState extends State<ValueEntryField> {
  final _controller = TextEditingController();
  String? _error;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _submit(String text) async {
    final value = parsePositiveNumber(text);
    if (value == null) {
      setState(() => _error = widget.notANumber);
      return;
    }
    final set = await widget.onSubmit(value);
    if (!mounted) return;
    setState(() => _error = set ? null : widget.outOfRange);
    if (set) _controller.clear();
  }

  @override
  Widget build(BuildContext context) => Row(
    children: [
      SizedBox(width: 120, child: Text(widget.label)),
      SizedBox(width: 88, child: Text(widget.currentText)),
      SizedBox(
        width: 200,
        child: TextField(
          controller: _controller,
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: InputDecoration(
            labelText: widget.fieldLabel,
            errorText: _error,
            isDense: true,
          ),
          onSubmitted: _submit,
        ),
      ),
      ?widget.trailing,
    ],
  );
}
