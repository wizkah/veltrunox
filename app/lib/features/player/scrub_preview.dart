import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'player_controller.dart';

/// A stopped recording that plays only while the dragged position rests. The
/// server really plays then, but this client keeps showing it as stopped.
class ScrubPreview extends Notifier<bool> {
  /// Longest wait for the server to confirm the pause after the release.
  static const _confirmWait = Duration(seconds: 1);

  Timer? _giveUp;
  bool _released = false;

  @override
  bool build() {
    ref.listen(playerStateProvider, (_, next) {
      if (_released && next.value?.running == false) _finish();
    });
    ref.onDispose(() => _giveUp?.cancel());
    return false;
  }

  /// Whether the drag that started the preview is still going on.
  bool get held => state && !_released;

  void start() {
    _giveUp?.cancel();
    _released = false;
    state = true;
  }

  /// The position was let go; the preview ends once the pause is confirmed.
  void release() {
    _released = true;
    _giveUp?.cancel();
    _giveUp = Timer(_confirmWait, _finish);
  }

  void _finish() {
    _giveUp?.cancel();
    _released = false;
    state = false;
  }
}

final scrubPreviewProvider = NotifierProvider<ScrubPreview, bool>(
  ScrubPreview.new,
);

/// Whether the player is shown as playing: not while only a preview plays.
final shownRunningProvider = Provider<bool>(
  (ref) =>
      (ref.watch(playerStateProvider).value?.running ?? false) &&
      !ref.watch(scrubPreviewProvider),
);
