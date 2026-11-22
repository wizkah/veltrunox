import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'connection_source.dart';
import 'connection_status.dart';

/// Supplies the [ConnectionSource]; overridden at the application root.
final connectionSourceProvider = Provider<ConnectionSource>(
  (ref) =>
      throw UnimplementedError('connectionSourceProvider must be overridden'),
);

/// Current connection status; "connecting" until the first report.
final connectionStatusProvider = StreamProvider<ConnectionStatus>((ref) async* {
  yield ConnectionStatus.connecting;
  yield* ref.watch(connectionSourceProvider).watch();
});

/// Turns the user's wish to connect again into a request to the source.
final connectionControllerProvider = Provider<ConnectionController>(
  (ref) => ConnectionController(ref.watch(connectionSourceProvider)),
);

class ConnectionController {
  ConnectionController(this._source);

  final ConnectionSource _source;

  Future<void> reconnect() => _source.reconnect();
}
