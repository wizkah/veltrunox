import 'package:material_ui/material_ui.dart';

import 'library_texts.dart';

/// Asks for a new title; resolves to it, or `null` when cancelled.
class RenameDialog extends StatefulWidget {
  const RenameDialog({required this.current, super.key});

  final String current;

  static Future<String?> ask(BuildContext context, String current) =>
      showDialog<String>(
        context: context,
        builder: (_) => RenameDialog(current: current),
      );

  @override
  State<RenameDialog> createState() => _RenameDialogState();
}

class _RenameDialogState extends State<RenameDialog> {
  late final _controller = TextEditingController(text: widget.current);
  bool _empty = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _submit() {
    final title = _controller.text.trim();
    if (title.isEmpty) {
      setState(() => _empty = true);
      return;
    }
    Navigator.of(context).pop(title);
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: const Text(LibraryTexts.renameTitle),
    content: TextField(
      controller: _controller,
      autofocus: true,
      decoration: InputDecoration(
        labelText: LibraryTexts.renameField,
        errorText: _empty ? LibraryTexts.renameEmpty : null,
      ),
      onSubmitted: (_) => _submit(),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.of(context).pop(),
        child: const Text(LibraryTexts.cancel),
      ),
      FilledButton(onPressed: _submit, child: const Text(LibraryTexts.save)),
    ],
  );
}
