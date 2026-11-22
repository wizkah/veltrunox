import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../shared_ui/toast_notice.dart';
import 'player_controller.dart';
import 'player_texts.dart';
import 'remote_change_detector.dart';

/// Shows, wherever the user is, what happens to the shared player without
/// them: refusals of the server and position changes made by others. The
/// notices do not block; [child] is shown unchanged.
class PlayerNotices extends ConsumerWidget {
  const PlayerNotices({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(playerRejectionProvider, (_, rejection) {
      final reason = rejection.value;
      if (reason != null) {
        ToastNotice.show(context, kind: ToastKind.error, message: reason);
      }
    });
    ref.listen(playerStateProvider, (_, next) {
      final state = next.value;
      if (state != null &&
          ref.read(remoteChangeDetectorProvider).isRemoteJump(state)) {
        ToastNotice.show(
          context,
          kind: ToastKind.info,
          message: PlayerTexts.positionChanged,
        );
      }
    });
    return child;
  }
}
