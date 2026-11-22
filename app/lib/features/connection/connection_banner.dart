import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'connection_controller.dart';
import 'connection_status.dart';
import 'connection_texts.dart';

/// A calm but conspicuous notice above the screens while the connection to the
/// server is lost, with the way to connect again. Nothing while connected.
class ConnectionBanner extends ConsumerWidget {
  const ConnectionBanner({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref.watch(connectionStatusProvider).value;
    if (status != ConnectionStatus.disconnected) return const SizedBox.shrink();
    return Semantics(
      liveRegion: true,
      child: Material(
        color: AppColors.surfaceRaised,
        shape: const Border(
          bottom: BorderSide(
            color: AppColors.error,
            width: AppSize.borderWidth,
          ),
        ),
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpace.s4,
            vertical: AppSpace.s2,
          ),
          child: Wrap(
            crossAxisAlignment: WrapCrossAlignment.center,
            spacing: AppSpace.s3,
            runSpacing: AppSpace.s2,
            children: [
              const Icon(Icons.cloud_off, color: AppColors.error),
              const Text(ConnectionTexts.lost),
              const Text(ConnectionTexts.lostHint),
              ActionButton(
                label: ConnectionTexts.reconnect,
                kind: ActionKind.secondary,
                icon: Icons.refresh,
                onPressed: ref.read(connectionControllerProvider).reconnect,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
