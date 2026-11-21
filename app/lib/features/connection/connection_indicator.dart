import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'connection_controller.dart';
import 'connection_status.dart';
import 'connection_texts.dart';

/// Small status display for the app bar.
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
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Center(child: Text(_label(status))),
    );
  }

  static String _label(ConnectionStatus status) => switch (status) {
    ConnectionStatus.connecting => ConnectionTexts.connecting,
    ConnectionStatus.connected => ConnectionTexts.connected,
    ConnectionStatus.disconnected => ConnectionTexts.disconnected,
  };
}
