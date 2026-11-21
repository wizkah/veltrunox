import '../features/connection/connection_source.dart';
import '../features/connection/connection_status.dart';
import 'generated/api.dart' as rust;

/// Adapter: reports the real-time channel state from the Rust client core.
class BridgeConnectionSource implements ConnectionSource {
  BridgeConnectionSource({required this.serverUrl});

  final String serverUrl;

  @override
  Stream<ConnectionStatus> watch() => rust
      .watchConnection(serverUrl: serverUrl)
      .map(
        (state) => switch (state) {
          rust.ConnectionState.connected => ConnectionStatus.connected,
          rust.ConnectionState.disconnected => ConnectionStatus.disconnected,
        },
      );
}
