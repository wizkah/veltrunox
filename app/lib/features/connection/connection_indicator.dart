import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'connection_controller.dart';
import 'connection_status.dart';
import 'connection_texts.dart';

/// Small status display for the shell's status area.
class ConnectionIndicator extends ConsumerWidget {
  const ConnectionIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref
        .watch(connectionStatusProvider)
        .maybeWhen(
          data: (status) => status,
          orElse: () => ConnectionStatus.disconnected,
        );
    final color = _color(status);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: AppSpace.s2),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.12),
          border: Border.all(color: color.withValues(alpha: 0.6)),
          borderRadius: BorderRadius.circular(AppRadius.pill),
        ),
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpace.s3,
            vertical: AppSpace.s1,
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(_icon(status), size: AppSpace.s4, color: color),
              const SizedBox(width: AppSpace.s2),
              Text(
                _label(status),
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ],
          ),
        ),
      ),
    );
  }

  static IconData _icon(ConnectionStatus status) => switch (status) {
    ConnectionStatus.connecting => Icons.sync,
    ConnectionStatus.connected => Icons.signal_cellular_alt,
    ConnectionStatus.disconnected => Icons.cloud_off,
  };

  static Color _color(ConnectionStatus status) => switch (status) {
    ConnectionStatus.connecting => AppColors.warning,
    ConnectionStatus.connected => AppColors.success,
    ConnectionStatus.disconnected => AppColors.error,
  };

  static String _label(ConnectionStatus status) => switch (status) {
    ConnectionStatus.connecting => ConnectionTexts.connecting,
    ConnectionStatus.connected => ConnectionTexts.connected,
    ConnectionStatus.disconnected => ConnectionTexts.disconnected,
  };
}
