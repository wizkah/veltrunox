import 'dart:async';

import '../features/connection/connection_status.dart';
import '../features/player/player_state.dart';
import 'generated/api.dart' as rust;

/// Adapter: the one real-time session of the Rust client core, shared by the
/// connection and player sources. Keeps the latest values so a feature that
/// starts listening late still sees the current state.
class BridgeSession {
  BridgeSession({required this.serverUrl}) {
    _subscription = rust.watchSession(serverUrl: serverUrl).listen(_onUpdate);
  }

  final String serverUrl;
  late final StreamSubscription<rust.SessionUpdate> _subscription;

  ConnectionStatus? _connection;
  PlayerState? _player;
  final _connections = StreamController<ConnectionStatus>.broadcast();
  final _players = StreamController<PlayerState>.broadcast();
  final _rejections = StreamController<String>.broadcast();

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

  /// Sends a command; a failure is reported like a refusal by the server.
  Future<void> send(rust.PlayerCommand command) async {
    try {
      await rust.sendCommand(command: command);
    } on Object catch (error) {
      _rejections.add(error.toString());
    }
  }

  Future<void> dispose() async {
    await _subscription.cancel();
    await _connections.close();
    await _players.close();
    await _rejections.close();
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
      case rust.SessionUpdate_BookmarksChanged():
        // The bookmark feature does not exist yet.
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
  );
}
