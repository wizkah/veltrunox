import 'package:material_ui/material_ui.dart';

/// A text field for a time such as "1:07" or "1:07,5".
class TimeField extends StatelessWidget {
  const TimeField({
    required this.controller,
    required this.label,
    required this.errorText,
    this.onSubmitted,
    this.textInputAction = TextInputAction.next,
    super.key,
  });

  final TextEditingController controller;
  final String label;
  final String? errorText;
  final VoidCallback? onSubmitted;
  final TextInputAction textInputAction;

  @override
  Widget build(BuildContext context) => TextField(
    controller: controller,
    decoration: InputDecoration(labelText: label, errorText: errorText),
    keyboardType: TextInputType.datetime,
    textInputAction: textInputAction,
    onSubmitted: onSubmitted == null ? null : (_) => onSubmitted!(),
  );
}
