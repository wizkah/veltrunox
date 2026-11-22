import 'connection_status.dart';

/// Port: reports the state of the real-time channel over time.
abstract interface class ConnectionSource {
  Stream<ConnectionStatus> watch();

  /// Opens the connection again at once instead of waiting for the next
  /// automatic attempt.
  Future<void> reconnect();
}
