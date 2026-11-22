import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'player_state.dart';

/// Tells a jump of the playback position made by another client from the
/// ordinary course of playback and from this client's own actions.
class RemoteChangeDetector {
  RemoteChangeDetector({DateTime Function()? now}) : _now = now ?? DateTime.now;

  /// How far the position may differ from the expected one before it counts
  /// as a jump (network delay, rounding).
  static const tolerance = 600;

  /// After an own action, states are answers to it for this long.
  static const ownWindow = Duration(seconds: 2);

  /// A jump is reported at most this often, so dragging by another client
  /// gives one notice, not a stream.
  static const quiet = Duration(seconds: 3);

  final DateTime Function() _now;
  PlayerState? _last;
  DateTime? _lastAt;
  DateTime? _ownAt;
  DateTime? _reportedAt;

  /// This client changed the position (or the recording) itself.
  void ownAction() => _ownAt = _now();

  /// Whether `state` moved the position without this client's doing.
  bool isRemoteJump(PlayerState state) {
    final before = _last;
    final beforeAt = _lastAt;
    final now = _now();
    _last = state;
    _lastAt = now;
    if (before == null ||
        beforeAt == null ||
        before.recordingId != state.recordingId) {
      return false;
    }
    final elapsed = now.difference(beforeAt).inMilliseconds;
    final expected = before.running
        ? before.positionMs + elapsed * before.speed
        : before.positionMs.toDouble();
    if ((state.positionMs - expected).abs() <= tolerance) return false;
    final own = _ownAt;
    if (own != null && now.difference(own) < ownWindow) return false;
    final reported = _reportedAt;
    if (reported != null && now.difference(reported) < quiet) return false;
    _reportedAt = now;
    return true;
  }
}

/// The one detector of this client; the player controller reports own actions
/// to it and the notices read from it.
final remoteChangeDetectorProvider = Provider<RemoteChangeDetector>(
  (ref) => RemoteChangeDetector(),
);
