import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'bridge/bridge_connection_source.dart';
import 'bridge/bridge_marker_source.dart';
import 'bridge/bridge_player_source.dart';
import 'bridge/bridge_recording_source.dart';
import 'bridge/bridge_session.dart';
import 'bridge/bridge_waveform_source.dart';
import 'bridge/generated/frb_generated.dart';
import 'features/connection/connection_controller.dart';
import 'features/library/library_controller.dart';
import 'features/markers/marker_controller.dart';
import 'features/player/player_controller.dart';
import 'features/waveform/waveform_source.dart';

const _configuredServerUrl = String.fromEnvironment('VELTRUNOX_SERVER_URL');

/// The server serves this app, so by default it is the origin the page came
/// from, whatever address and port the server was started with. A separately
/// served app (for example `flutter run`) names its server at build time.
String _resolveServerUrl() {
  if (_configuredServerUrl.isNotEmpty) {
    return _configuredServerUrl;
  }
  return Uri.base.origin;
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
        markerSourceProvider.overrideWithValue(
          BridgeMarkerSource(serverUrl: serverUrl, session: session),
        ),
        waveformSourceProvider.overrideWithValue(
          BridgeWaveformSource(serverUrl: serverUrl),
        ),
        recordingSourceProvider.overrideWithValue(
          BridgeRecordingSource(serverUrl: serverUrl, session: session),
        ),
      ],
      child: const VeltrunoxApp(),
    ),
  );
}
