import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';

enum ToastKind { info, success, warning, error }

/// A short message that does not block anything. The kind shows as an icon too,
/// so it is not conveyed by colour alone.
class ToastNotice extends StatelessWidget {
  const ToastNotice({required this.kind, required this.message, super.key});

  final ToastKind kind;
  final String message;

  /// Shows the notice as a non-blocking snack bar.
  static void show(
    BuildContext context, {
    required ToastKind kind,
    required String message,
  }) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: ToastNotice(kind: kind, message: message),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => Row(
    children: [
      Icon(_icon, color: _color, semanticLabel: null),
      const SizedBox(width: AppSpace.s3),
      Expanded(child: Text(message)),
    ],
  );

  IconData get _icon => switch (kind) {
    ToastKind.info => Icons.info_outline,
    ToastKind.success => Icons.check_circle_outline,
    ToastKind.warning => Icons.warning_amber,
    ToastKind.error => Icons.error_outline,
  };

  Color get _color => switch (kind) {
    ToastKind.info => AppColors.accentSoft,
    ToastKind.success => AppColors.success,
    ToastKind.warning => AppColors.warning,
    ToastKind.error => AppColors.error,
  };
}
