import 'dart:async';

/// What happens when a position is dragged: moving it scrubs (a short piece is
/// heard), holding it still for [holdDelay] lets the recording play on from
/// there, letting go ends the drag. Shared by every control that moves the
/// position, so they all behave alike.
class ScrubHold {
  ScrubHold({
    required this.onScrub,
    required this.onScrubEnd,
    required this.onHold,
    required this.onHoldEnd,
    this.holdDelay = const Duration(milliseconds: 130),
  });

  final void Function(int positionMs) onScrub;
  final void Function(int positionMs) onScrubEnd;

  /// The position rests: the recording plays on from there.
  final void Function(int positionMs) onHold;

  /// The position is let go after it rested.
  final void Function(int positionMs) onHoldEnd;
  final Duration holdDelay;

  Timer? _hold;

  /// The recording already plays on from the held position; the release must
  /// not move the playback back.
  bool _playingOn = false;

  /// The position was moved to [positionMs].
  void moved(int positionMs) {
    _hold?.cancel();
    _playingOn = false;
    onScrub(positionMs);
    _hold = Timer(holdDelay, () {
      _playingOn = true;
      onHold(positionMs);
    });
  }

  /// The position was let go at [positionMs].
  void released(int positionMs) {
    _hold?.cancel();
    if (_playingOn) {
      onHoldEnd(positionMs);
    } else {
      onScrubEnd(positionMs);
    }
    _playingOn = false;
  }

  void dispose() => _hold?.cancel();
}
