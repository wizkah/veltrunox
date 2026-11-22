import 'dart:async';

import '../features/connection/connection_status.dart';
import '../features/player/player_state.dart';
import 'generated/api.dart' as rust;

/// Adapter: the one real-time session of the Rust client core, shared by the
/// connection and player sources. Keeps the latest values so a feature that
/// starts listening late still sees the current state.
class BridgeSession {
  BridgeSession({required this.serverUrl});

  final String serverUrl;
  StreamSubscription<rust.SessionUpdate>? _subscription;

  ConnectionStatus? _connection;
  PlayerState? _player;
  final _connections = StreamController<ConnectionStatus>.broadcast();
  final _players = StreamController<PlayerState>.broadcast();
  final _rejections = StreamController<String>.broadcast();
  final _analysisChanges = StreamController<int>.broadcast();
  final _bookmarkChanges = StreamController<int>.broadcast();

  Stream<ConnectionStatus> connection() async* {
    final latest = _connection;
    if (latest != null) yield latest;
    yield* _connections.stream;
  }

  Stream<PlayerState> player() async* {
    final latest = _player;
    if (latest != null) yield latest;
    yield* _players.stream;
  }

  Stream<String> rejections() => _rejections.stream;

  /// Ids of recordings whose analysis changed on the server.
  Stream<int> analysisChanges() => _analysisChanges.stream;

  /// Ids of recordings whose markers changed on the server.
  Stream<int> bookmarkChanges() => _bookmarkChanges.stream;

  /// Replaces the channel by a new one that connects at once. The shared
  /// state arrives again with the first message of the new connection.
  Future<void> reconnect() => open();

  /// Opens the session with the access the Rust client core holds now,
  /// replacing an open one. Call it after signing in.
  Future<void> open() async {
    await _subscription?.cancel();
    _connection = null;
    _player = null;
    _subscription = _open();
  }

  /// Stops listening, for example when signing out.
  Future<void> close() async {
    await _subscription?.cancel();
    _subscription = null;
    _connection = null;
    _player = null;
  }

  StreamSubscription<rust.SessionUpdate> _open() =>
      rust.watchSession(serverUrl: serverUrl).listen(_onUpdate);

  /// Sends a command; a failure is reported like a refusal by the server.
  Future<void> send(rust.PlayerCommand command) async {
    try {
      await rust.sendCommand(command: command);
    } on Object catch (error) {
      _rejections.add(error.toString());
    }
  }

  Future<void> dispose() async {
    await _subscription?.cancel();
    await _connections.close();
    await _players.close();
    await _rejections.close();
    await _analysisChanges.close();
    await _bookmarkChanges.close();
  }

  void _onUpdate(rust.SessionUpdate update) {
    switch (update) {
      case rust.SessionUpdate_Connection(:final field0):
        _onConnection(field0);
      case rust.SessionUpdate_Playback(:final field0):
        _player = _playerState(field0);
        _players.add(_player!);
      case rust.SessionUpdate_Rejected(:final reason):
        _rejections.add(reason);
      case rust.SessionUpdate_AnalysisChanged(:final recordingId):
        _analysisChanges.add(recordingId.toInt());
      case rust.SessionUpdate_BookmarksChanged(:final recordingId):
        _bookmarkChanges.add(recordingId.toInt());
      case rust.SessionUpdate_Rights():
        // The interface shows sign-in and rights in a later step.
        break;
    }
  }

  void _onConnection(rust.ConnectionState state) {
    _connection = switch (state) {
      rust.ConnectionState.connected => ConnectionStatus.connected,
      rust.ConnectionState.disconnected => ConnectionStatus.disconnected,
    };
    _connections.add(_connection!);
  }

  static PlayerState _playerState(rust.PlaybackView view) => PlayerState(
    recordingId: view.recordingId?.toInt(),
    running: view.running,
    speed: view.speed,
    tuningHz: view.tuningHz,
    keyShiftSemitones: view.keyShiftSemitones,
    volumePercent: view.volumePercent,
    positionMs: view.positionMs.toInt(),
    playMode: switch (view.playMode) {
      rust.PlayModeChoice.toEnd => PlayMode.toEnd,
      rust.PlayModeChoice.toRegionEnd => PlayMode.toRegionEnd,
      rust.PlayModeChoice.loop => PlayMode.loop,
    },
    playRange: view.playRange.let(
      (range) => (startMs: range.startMs.toInt(), endMs: range.endMs.toInt()),
    ),
  );
}

extension _Let<T> on T? {
  R? let<R>(R Function(T) transform) {
    final value = this;
    return value == null ? null : transform(value);
  }
}
