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

const _serverUrl = String.fromEnvironment(
  'VELTRUNOX_SERVER_URL',
  defaultValue: 'http://127.0.0.1:8080',
);

Future<void> main() async {
  await RustLib.init();
  final session = BridgeSession(serverUrl: _serverUrl);
  runApp(
    ProviderScope(
      overrides: [
        connectionSourceProvider.overrideWithValue(
          BridgeConnectionSource(session),
        ),
        playerSourceProvider.overrideWithValue(BridgePlayerSource(session)),
        recordingSourceProvider.overrideWithValue(
          BridgeRecordingSource(serverUrl: _serverUrl),
        ),
      ],
      child: const VeltrunoxApp(),
    ),
  );
}
