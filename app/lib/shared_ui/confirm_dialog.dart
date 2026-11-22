import 'package:material_ui/material_ui.dart';

import 'action_button.dart';

/// Asks for explicit confirmation, for example before deleting. Only for
/// consequential actions, never for ordinary playback.
class ConfirmDialog extends StatelessWidget {
  const ConfirmDialog({
    required this.title,
    required this.message,
    required this.confirmLabel,
    required this.cancelLabel,
    this.destructive = true,
    super.key,
  });

  final String title;
  final String message;
  final String confirmLabel;
  final String cancelLabel;
  final bool destructive;

  /// Resolves to `true` only when the user confirmed.
  static Future<bool> ask(
    BuildContext context, {
    required String title,
    required String message,
    required String confirmLabel,
    required String cancelLabel,
    bool destructive = true,
  }) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (_) => ConfirmDialog(
        title: title,
        message: message,
        confirmLabel: confirmLabel,
        cancelLabel: cancelLabel,
        destructive: destructive,
      ),
    );
    return confirmed ?? false;
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text(title),
    content: Text(message),
    actions: [
      ActionButton(
        label: cancelLabel,
        kind: ActionKind.secondary,
        onPressed: () => Navigator.of(context).pop(false),
      ),
      ActionButton(
        label: confirmLabel,
        kind: destructive ? ActionKind.danger : ActionKind.primary,
        onPressed: () => Navigator.of(context).pop(true),
      ),
    ],
  );
}
