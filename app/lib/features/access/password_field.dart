import 'package:material_ui/material_ui.dart';

import 'access_texts.dart';

/// A password input that is hidden until the person asks to see it, and
/// hidden again when it is left.
class PasswordField extends StatefulWidget {
  const PasswordField({
    required this.controller,
    required this.label,
    this.error,
    this.onSubmitted,
    this.textInputAction,
    super.key,
  });

  final TextEditingController controller;
  final String label;
  final String? error;
  final VoidCallback? onSubmitted;
  final TextInputAction? textInputAction;

  @override
  State<PasswordField> createState() => _PasswordFieldState();
}

class _PasswordFieldState extends State<PasswordField> {
  bool _visible = false;

  @override
  Widget build(BuildContext context) => TextField(
    controller: widget.controller,
    obscureText: !_visible,
    autocorrect: false,
    enableSuggestions: false,
    textInputAction: widget.textInputAction,
    onSubmitted: widget.onSubmitted == null
        ? null
        : (_) => widget.onSubmitted!(),
    decoration: InputDecoration(
      labelText: widget.label,
      errorText: widget.error,
      suffixIcon: IconButton(
        tooltip: _visible ? AccessTexts.hidePassword : AccessTexts.showPassword,
        icon: Icon(_visible ? Icons.visibility_off : Icons.visibility),
        onPressed: () => setState(() => _visible = !_visible),
      ),
    ),
  );
}
