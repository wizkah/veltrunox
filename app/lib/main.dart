import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'bridge/bridge_connection_source.dart';
import 'bridge/bridge_recording_source.dart';
import 'bridge/generated/frb_generated.dart';
import 'features/connection/connection_controller.dart';
import 'features/library/library_controller.dart';

const _serverUrl = String.fromEnvironment(
  'VELTRUNOX_SERVER_URL',
  defaultValue: 'http://127.0.0.1:8080',
);

Future<void> main() async {
  await RustLib.init();
  runApp(
    ProviderScope(
      overrides: [
        connectionSourceProvider.overrideWithValue(
          BridgeConnectionSource(serverUrl: _serverUrl),
        ),
        recordingSourceProvider.overrideWithValue(
          BridgeRecordingSource(serverUrl: _serverUrl),
        ),
      ],
      child: const VeltrunoxApp(),
    ),
  );
}
