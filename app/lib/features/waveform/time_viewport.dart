import 'dart:math' as math;

/// The visible time range of a recording on screen. Local view state only:
/// it never changes the shared playback position.
///
/// All operations return a new, always valid viewport (clamped to the
/// recording and to the zoom limits).
final class TimeViewport {
  const TimeViewport._({
    required this.durationMs,
    required this.widthPx,
    required this.startMs,
    required this.msPerPx,
  });

  /// Finest zoom: one millisecond per pixel.
  static const minMsPerPx = 1.0;

  /// A viewport showing the whole recording.
  factory TimeViewport.fit({required int durationMs, required double widthPx}) {
    final safeWidth = math.max(widthPx, 1.0);
    return TimeViewport._(
      durationMs: durationMs,
      widthPx: safeWidth,
      startMs: 0,
      msPerPx: _fitMsPerPx(durationMs, safeWidth),
    );
  }

  final int durationMs;
  final double widthPx;
  final double startMs;
  final double msPerPx;

  double get visibleSpanMs => widthPx * msPerPx;
  double get endMs => startMs + visibleSpanMs;

  bool get isFitted => msPerPx >= _fitMsPerPx(durationMs, widthPx);

  double timeToX(double ms) => (ms - startMs) / msPerPx;
  double xToTime(double x) => startMs + x * msPerPx;

  bool isVisible(double ms) => ms >= startMs && ms <= endMs;

  /// Zooms by [factor] (above 1 zooms in) and keeps the time under [anchorX]
  /// at the same pixel.
  TimeViewport zoomedAt(double factor, double anchorX) {
    final anchorMs = xToTime(anchorX);
    final newMsPerPx = _clampMsPerPx(msPerPx / factor);
    return _clamped(anchorMs - anchorX * newMsPerPx, newMsPerPx);
  }

  /// Zoom for the `+`/`−` buttons: around the playhead if it is in view,
  /// otherwise around the centre of the viewport.
  TimeViewport zoomedForButton(double factor, {required double playheadMs}) {
    final anchorX = isVisible(playheadMs) ? timeToX(playheadMs) : widthPx / 2;
    return zoomedAt(factor, anchorX);
  }

  /// Moves the content with a drag of [deltaPx] (dragging right shows earlier
  /// time).
  TimeViewport pannedByPixels(double deltaPx) =>
      _clamped(startMs - deltaPx * msPerPx, msPerPx);

  /// Keeps the visible time range when the widget is resized; a view of the
  /// whole recording stays one, and the zoom never goes below it.
  TimeViewport withWidth(double newWidthPx) {
    final safeWidth = math.max(newWidthPx, 1.0);
    if (isFitted) {
      return TimeViewport.fit(durationMs: durationMs, widthPx: safeWidth);
    }
    final resized = TimeViewport._(
      durationMs: durationMs,
      widthPx: safeWidth,
      startMs: startMs,
      msPerPx: msPerPx,
    );
    return resized._clamped(startMs, resized._clampMsPerPx(msPerPx));
  }

  TimeViewport fitted() =>
      TimeViewport.fit(durationMs: durationMs, widthPx: widthPx);

  /// Scrolls the minimum needed to show the range; zooms out only if the range
  /// is longer than the view. Used to show a region opened from the list.
  TimeViewport revealing(double fromMs, double toMs) {
    if (fromMs >= startMs && toMs <= endMs) return this;
    final span = toMs - fromMs;
    final newMsPerPx = span > visibleSpanMs
        ? _clampMsPerPx(span / widthPx)
        : msPerPx;
    final newSpan = widthPx * newMsPerPx;
    final newStart = fromMs < startMs ? fromMs : toMs - newSpan;
    return _clamped(newStart, newMsPerPx);
  }

  TimeViewport _clamped(double newStartMs, double newMsPerPx) {
    final span = widthPx * newMsPerPx;
    final maxStart = math.max(0.0, durationMs - span);
    return TimeViewport._(
      durationMs: durationMs,
      widthPx: widthPx,
      startMs: newStartMs.clamp(0.0, maxStart),
      msPerPx: newMsPerPx,
    );
  }

  double _clampMsPerPx(double value) => value.clamp(
    math.min(minMsPerPx, _fitMsPerPx(durationMs, widthPx)),
    _fitMsPerPx(durationMs, widthPx),
  );

  static double _fitMsPerPx(int durationMs, double widthPx) =>
      math.max(durationMs / widthPx, 1e-6);

  @override
  bool operator ==(Object other) =>
      other is TimeViewport &&
      other.durationMs == durationMs &&
      other.widthPx == widthPx &&
      other.startMs == startMs &&
      other.msPerPx == msPerPx;

  @override
  int get hashCode => Object.hash(durationMs, widthPx, startMs, msPerPx);
}
