import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/app.dart';
import 'package:veltrunox_app/features/connection/connection_controller.dart';
import 'package:veltrunox_app/features/connection/connection_source.dart';
import 'package:veltrunox_app/features/connection/connection_status.dart';
import 'package:veltrunox_app/features/library/library_controller.dart';
import 'package:veltrunox_app/features/library/recording_source.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';

class _FakeSource implements RecordingSource {
  _FakeSource(this.result);

  final Future<List<RecordingSummary>> Function() result;

  @override
  Future<List<RecordingSummary>> listRecordings() => result();
}

class _FakeConnection implements ConnectionSource {
  @override
  Stream<ConnectionStatus> watch() => Stream.value(ConnectionStatus.connected);
}

Widget _app(RecordingSource source) => ProviderScope(
  overrides: [
    recordingSourceProvider.overrideWithValue(source),
    connectionSourceProvider.overrideWithValue(_FakeConnection()),
  ],
  child: const VeltrunoxApp(),
);

void main() {
  testWidgets('shows the connection status', (tester) async {
    await tester.pumpWidget(_app(_FakeSource(() async => [])));
    await tester.pumpAndSettle();

    expect(find.text('Verbunden'), findsOneWidget);
  });

  testWidgets('shows the recordings from the source', (tester) async {
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [const RecordingSummary(id: 1, title: 'Soundcheck')],
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Soundcheck'), findsOneWidget);
  });

  testWidgets('shows a retry action when loading fails', (tester) async {
    await tester.pumpWidget(
      _app(_FakeSource(() async => throw Exception('down'))),
    );
    await tester.pumpAndSettle();

    expect(find.text('Erneut versuchen'), findsOneWidget);
  });
}
