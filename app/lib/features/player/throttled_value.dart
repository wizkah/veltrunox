import 'dart:async';

/// Passes on at most one value per [interval], the first at once and the latest
/// one when the interval is over, so the last value is never lost.
class ThrottledValue<T> {
  ThrottledValue(
    this._send, {
    this.interval = const Duration(milliseconds: 60),
  });

  final void Function(T value) _send;
  final Duration interval;

  Timer? _timer;
  T? _waiting;
  bool _hasWaiting = false;

  void add(T value) {
    if (_timer != null) {
      _waiting = value;
      _hasWaiting = true;
      return;
    }
    _send(value);
    _timer = Timer(interval, _release);
  }

  void _release() {
    _timer = null;
    if (!_hasWaiting) return;
    final value = _waiting as T;
    _waiting = null;
    _hasWaiting = false;
    add(value);
  }

  /// Stops a pending delivery, for example when the owner goes away.
  void dispose() {
    _timer?.cancel();
    _timer = null;
    _hasWaiting = false;
  }
}
