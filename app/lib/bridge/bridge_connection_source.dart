import '../features/connection/connection_source.dart';
import '../features/connection/connection_status.dart';
import 'bridge_session.dart';

/// Adapter: reports the real-time channel state from the Rust client core.
class BridgeConnectionSource implements ConnectionSource {
  BridgeConnectionSource(this._session);

  final BridgeSession _session;

  @override
  Stream<ConnectionStatus> watch() => _session.connection();

  @override
  Future<void> reconnect() => _session.reconnect();
}
