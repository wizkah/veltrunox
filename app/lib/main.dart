import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'bridge/bridge_connection_source.dart';
import 'bridge/bridge_player_source.dart';
import 'bridge/bridge_recording_source.dart';
import 'bridge/bridge_session.dart';
import 'bridge/generated/frb_generated.dart';
import 'features/connection/connection_controller.dart';
import 'features/library/library_controller.dart';
import 'features/player/player_controller.dart';

const _configuredServerUrl = String.fromEnvironment('VELTRUNOX_SERVER_URL');
const _serverPort = 8080;

/// The configured server, or else the server on the host this page came from,
/// so `localhost` and `127.0.0.1` each reach the server by the same name.
String _resolveServerUrl() {
  if (_configuredServerUrl.isNotEmpty) {
    return _configuredServerUrl;
  }
  final page = Uri.base;
  final host = page.host.isEmpty ? '127.0.0.1' : page.host;
  return 'http://$host:$_serverPort';
}

Future<void> main() async {
  await RustLib.init();
  final serverUrl = _resolveServerUrl();
  final session = BridgeSession(serverUrl: serverUrl);
  runApp(
    ProviderScope(
      overrides: [
        connectionSourceProvider.overrideWithValue(
          BridgeConnectionSource(session),
        ),
        playerSourceProvider.overrideWithValue(BridgePlayerSource(session)),
        recordingSourceProvider.overrideWithValue(
          BridgeRecordingSource(serverUrl: serverUrl),
        ),
      ],
      child: const VeltrunoxApp(),
    ),
  );
}
