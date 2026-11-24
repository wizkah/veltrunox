/// The position an action here has asked for, shown until the server's
/// position has caught up with it. Without it the position falls back to the
/// old value after a click and only then jumps to the new one, as long as the
/// answer of the server takes.
class PendingPosition {
  PendingPosition({
    Duration Function()? clock,
    this.tolerance = const Duration(milliseconds: 500),
    this.timeout = const Duration(milliseconds: 1500),
  }) : _clock = clock ?? _stopwatchClock();

  final Duration Function() _clock;

  /// How far the position of the server may be from the asked one and still
  /// count as arrived.
  final Duration tolerance;

  /// The longest the asked position is shown without the server confirming it.
  final Duration timeout;

  int? _targetMs;
  Duration _askedAt = Duration.zero;

  /// An action moved the position to [positionMs]; replaces an earlier one.
  void expect(int positionMs) {
    _targetMs = positionMs;
    _askedAt = _clock();
  }

  /// The position to show, given what the server reports ([serverMs]). While
  /// playing ([running] at [speed]) the asked position moves on with the
  /// recording.
  int? shown(int? serverMs, {required bool running, required double speed}) {
    final target = _targetMs;
    if (target == null) return serverMs;
    final elapsed = _clock() - _askedAt;
    if (elapsed > timeout) {
      _targetMs = null;
      return serverMs;
    }
    final advance = running ? (elapsed.inMilliseconds * speed).round() : 0;
    final expected = target + advance;
    if (serverMs != null &&
        serverMs >= target - tolerance.inMilliseconds &&
        serverMs <= expected + tolerance.inMilliseconds) {
      _targetMs = null;
      return serverMs;
    }
    return expected;
  }

  static Duration Function() _stopwatchClock() {
    final watch = Stopwatch()..start();
    return () => watch.elapsed;
  }
}
