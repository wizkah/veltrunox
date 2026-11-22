import 'package:material_ui/material_ui.dart';

import 'number_input.dart';

/// One field that shows a value and takes a wanted number for it. The field
/// follows the value as it stands; typing a number and confirming (Enter) hands
/// it on, and a message shows when it was not taken. Leaving the field with
/// something typed puts the current value back.
class ValueEntryField extends StatefulWidget {
  const ValueEntryField({
    required this.label,
    required this.valueText,
    required this.unit,
    required this.notANumber,
    required this.outOfRange,
    required this.onSubmit,
    this.trailing,
    this.enabled = true,
    super.key,
  });

  final String label;

  /// The value as it stands now, without its unit.
  final String valueText;
  final String unit;

  /// Messages for text that is no number and for a number that was not taken.
  final String notANumber;
  final String outOfRange;

  /// Reports whether the number could be set.
  final Future<bool> Function(double value) onSubmit;

  /// Shown after the field, for example a reset button.
  final Widget? trailing;

  /// A disabled field shows the value but takes no input.
  final bool enabled;

  @override
  State<ValueEntryField> createState() => _ValueEntryFieldState();
}

class _ValueEntryFieldState extends State<ValueEntryField> {
  late final _controller = TextEditingController(text: widget.valueText);
  late final _focus = FocusNode()..addListener(_onFocusChange);
  String? _error;

  /// Something was typed that has not been handed on.
  bool _typed = false;

  @override
  void didUpdateWidget(ValueEntryField old) {
    super.didUpdateWidget(old);
    if (!_typed && _controller.text != widget.valueText) {
      _controller.text = widget.valueText;
    }
  }

  @override
  void dispose() {
    _focus.dispose();
    _controller.dispose();
    super.dispose();
  }

  void _onFocusChange() {
    // A message stays, with what was typed, until the user changes it.
    if (_focus.hasFocus || !_typed || _error != null) return;
    setState(_putCurrentValueBack);
  }

  void _putCurrentValueBack() {
    _typed = false;
    _error = null;
    _controller.text = widget.valueText;
  }

  Future<void> _submit(String text) async {
    final value = parsePositiveNumber(text);
    if (value == null) {
      setState(() => _error = widget.notANumber);
      return;
    }
    final set = await widget.onSubmit(value);
    if (!mounted) return;
    setState(() {
      if (set) {
        _putCurrentValueBack();
      } else {
        _error = widget.outOfRange;
      }
    });
  }

  @override
  Widget build(BuildContext context) => Wrap(
    crossAxisAlignment: WrapCrossAlignment.center,
    children: [
      SizedBox(width: 152, child: Text(widget.label)),
      SizedBox(
        width: 160,
        child: Semantics(
          label: widget.label,
          child: TextField(
            controller: _controller,
            focusNode: _focus,
            enabled: widget.enabled,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            decoration: InputDecoration(
              suffixText: widget.unit,
              hintText: '–',
              errorText: _error,
              isDense: true,
            ),
            onChanged: (text) {
              _typed = text != widget.valueText;
              if (_error != null) setState(() => _error = null);
            },
            onSubmitted: _submit,
          ),
        ),
      ),
      ?widget.trailing,
    ],
  );
}
