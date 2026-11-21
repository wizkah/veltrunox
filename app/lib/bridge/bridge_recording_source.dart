import '../features/library/recording_source.dart';
import '../features/library/recording_summary.dart';
import 'generated/api.dart' as rust;

/// Adapter: reads the recording list through the Rust client core.
class BridgeRecordingSource implements RecordingSource {
  BridgeRecordingSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<List<RecordingSummary>> listRecordings() async {
    final items = await rust.listRecordings(serverUrl: serverUrl);
    return [
      for (final item in items)
        RecordingSummary(id: item.id.toInt(), title: item.title),
    ];
  }
}
