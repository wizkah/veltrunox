import 'dart:ui' show PlatformDispatcher;

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'bridge/bridge_access_source.dart';
import 'bridge/bridge_connection_source.dart';
import 'bridge/bridge_exchange_source.dart';
import 'bridge/bridge_marker_source.dart';
import 'bridge/bridge_player_source.dart';
import 'bridge/bridge_recording_source.dart';
import 'bridge/bridge_own_password_source.dart';
import 'bridge/bridge_token_source.dart';
import 'bridge/bridge_session.dart';
import 'bridge/bridge_sharing_source.dart';
import 'bridge/bridge_user_source.dart';
import 'bridge/bridge_waveform_source.dart';
import 'bridge/generated/host_api.dart';
import 'composition/local_owner.dart';
import 'features/access/access_controller.dart';
import 'features/access/account_menu.dart';
import 'features/connection/connection_controller.dart';
import 'features/exchange/exchange_source.dart';
import 'features/exchange/file_exchange.dart';
import 'features/language/language_controller.dart';
import 'features/library/audio_file_picker.dart';
import 'features/library/library_controller.dart';
import 'features/markers/marker_controller.dart';
import 'features/player/player_controller.dart';
import 'features/own_password/own_password_source.dart';
import 'features/server_sharing/sharing_controller.dart';
import 'features/tokens/token_source.dart';
import 'features/users/users_controller.dart';
import 'features/waveform/waveform_source.dart';
import 'platform/clipboard_adapter.dart';
import 'platform/file_exchange_adapter.dart';
import 'platform/file_picker_adapter.dart';
import 'platform/rust_library.dart';
import 'shared_ui/app_language.dart';
import 'storage/shared_preferences_language_store.dart';
import 'storage/shared_preferences_token_store.dart';

const _configuredServerUrl = String.fromEnvironment('VELTRUNOX_SERVER_URL');

/// In a browser the server serves this app, so by default it is the origin the page came
/// from, whatever address and port the server was started with. A separately
/// served app (for example `flutter run`) names its server at build time.
String _resolveServerUrl() {
  if (_configuredServerUrl.isNotEmpty) {
    return _configuredServerUrl;
  }
  return Uri.base.origin;
}

Future<void> main() async {
  await initRustLibrary();
  // The desktop application carries its server; a build can still name
  // another one for development.
  final host = !kIsWeb && _configuredServerUrl.isEmpty
      ? await startLocalHost()
      : null;
  final serverUrl = host?.url ?? _resolveServerUrl();
  final session = BridgeSession(serverUrl: serverUrl);
  final languageStore = SharedPreferencesLanguageStore();
  // The language chosen earlier, else the one of the device, else English.
  final language =
      await languageStore.read() ??
      AppLanguage.fromCode(PlatformDispatcher.instance.locale.languageCode) ??
      AppLanguage.en;
  runApp(
    ProviderScope(
      overrides: [
        languageStoreProvider.overrideWithValue(languageStore),
        initialLanguageProvider.overrideWithValue(language),
        accessSourceProvider.overrideWithValue(
          host == null
              ? BridgeAccessSource(serverUrl: serverUrl, session: session)
              : OwnerAccessSource(
                  BridgeAccessSource(serverUrl: serverUrl, session: session),
                ),
        ),
        tokenStoreProvider.overrideWithValue(
          host == null
              ? SharedPreferencesTokenStore()
              : OwnerTokenStore(host.ownerToken),
        ),
        localOwnerProvider.overrideWithValue(host != null),
        sharingSourceProvider.overrideWithValue(
          host == null ? null : BridgeSharingSource(),
        ),
        userSourceProvider.overrideWithValue(
          BridgeUserSource(serverUrl: serverUrl),
        ),
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
        audioFilePickerProvider.overrideWithValue(SystemAudioFilePicker()),
        exchangeSourceProvider.overrideWithValue(
          BridgeExchangeSource(serverUrl: serverUrl),
        ),
        tokenSourceProvider.overrideWithValue(
          BridgeTokenSource(serverUrl: serverUrl),
        ),
        ownPasswordSourceProvider.overrideWithValue(
          BridgeOwnPasswordSource(serverUrl: serverUrl),
        ),
        textClipboardProvider.overrideWithValue(SystemTextClipboard()),
        textFilePickerProvider.overrideWithValue(SystemTextFilePicker()),
        fileSaverProvider.overrideWithValue(SystemFileSaver()),
        recordingSourceProvider.overrideWithValue(
          BridgeRecordingSource(serverUrl: serverUrl, session: session),
        ),
      ],
      child: const VeltrunoxApp(),
    ),
  );
}
