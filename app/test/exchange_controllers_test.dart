import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/exchange/exchange_models.dart';
import 'package:veltrunox_app/features/exchange/exchange_source.dart';
import 'package:veltrunox_app/features/exchange/export_controller.dart';
import 'package:veltrunox_app/features/exchange/file_exchange.dart';
import 'package:veltrunox_app/features/exchange/import_controller.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

import 'fake_exchange.dart';

const _ready = RecordingImport(
  recordingId: 1,
  status: ImportStatus.ready,
  added: 2,
  changed: 1,
  unchanged: 3,
  removed: 4,
);
const _conflict = RecordingImport(
  recordingId: 2,
  status: ImportStatus.conflict,
  issues: [
    ImportIssue(
      kind: ImportEntryKind.label,
      index: 0,
      reason: ServerReason('bookmark_conflict', args: [3]),
    ),
  ],
);

ProviderContainer _container({
  FakeExchangeSource? source,
  FakeFileSaver? saver,
  FakeTextFilePicker? picker,
}) {
  final container = ProviderContainer(
    overrides: [
      exchangeSourceProvider.overrideWithValue(source ?? FakeExchangeSource()),
      fileSaverProvider.overrideWithValue(saver ?? FakeFileSaver()),
      textFilePickerProvider.overrideWithValue(picker ?? FakeTextFilePicker()),
    ],
  );
  addTearDown(container.dispose);
  return container;
}

void main() {
  group('formats and reports', () {
    test('a file name names its format by the extension', () {
      expect(ExchangeFormat.fromFileName('a.JSON'), ExchangeFormat.json);
      expect(
        ExchangeFormat.fromFileName('Song.bookmarks.csv'),
        ExchangeFormat.csv,
      );
      expect(ExchangeFormat.fromFileName('notes.txt'), isNull);
      expect(ExchangeFormat.fromFileName('csv'), isNull);
    });

    test('the report counts ready recordings and what they remove', () {
      const report = ImportReport(
        dryRun: true,
        recordings: [
          _ready,
          _conflict,
          RecordingImport(
            recordingId: 3,
            status: ImportStatus.ready,
            removed: 1,
          ),
        ],
      );
      expect(report.readyCount, 2);
      expect(report.leftAsIsCount, 1);
      expect(report.hasConflicts, isTrue);
      expect(report.removedCount, 5);
    });

    test(
      'the file name follows the recording and avoids forbidden characters',
      () {
        const song = ExportScope.recording(id: 1, title: 'AC/DC: Live?');
        expect(
          exportFileName(song, ExchangeFormat.json),
          'AC_DC_ Live_.bookmarks.json',
        );
        expect(
          exportFileName(const ExportScope.library(), ExchangeFormat.csv),
          'veltrunox-bookmarks.csv',
        );
      },
    );
  });

  group('export', () {
    test('reads the bookmarks and saves them under the file name', () async {
      final source = FakeExchangeSource(exported: 'the text');
      final saver = FakeFileSaver();
      final container = _container(source: source, saver: saver);
      final state = await container
          .read(exportControllerProvider.notifier)
          .export(
            const ExportScope.recording(id: 7, title: 'Song'),
            ExchangeFormat.csv,
          );
      expect(state, isA<ExportSaved>());
      expect(source.calls, ['export 7 csv']);
      expect(saver.saved.single.fileName, 'Song.bookmarks.csv');
      expect(saver.saved.single.content, 'the text');
      expect(saver.saved.single.mimeType, 'text/csv');
      expect(exportNotice(state), contains('Song.bookmarks.csv'));
    });

    test('a closed save dialog is no error and no notice', () async {
      final saver = FakeFileSaver()..cancel = true;
      final container = _container(saver: saver);
      final state = await container
          .read(exportControllerProvider.notifier)
          .export(const ExportScope.library(), ExchangeFormat.json);
      expect(state, isA<ExportCancelled>());
      expect(exportNotice(state), isNull);
    });

    test('a refusal of the server is reported in words', () async {
      final source = FakeExchangeSource()..failure = const NotAllowed();
      final container = _container(source: source);
      final state = await container
          .read(exportControllerProvider.notifier)
          .export(const ExportScope.library(), ExchangeFormat.json);
      expect(state, isA<ExportFailed>());
      expect(exportNotice(state), contains('Berechtigung'));
    });

    test('a device that cannot save is reported', () async {
      final saver = FakeFileSaver()..failure = 'disk full';
      final container = _container(saver: saver);
      final state = await container
          .read(exportControllerProvider.notifier)
          .export(const ExportScope.library(), ExchangeFormat.json);
      expect(exportNotice(state), contains('disk full'));
    });
  });

  group('import', () {
    FakeExchangeSource source() => FakeExchangeSource()
      ..report = const ImportReport(
        dryRun: true,
        recordings: [_ready, _conflict],
      )
      ..reportWithOverwrite = const ImportReport(
        dryRun: true,
        recordings: [
          _ready,
          RecordingImport(
            recordingId: 2,
            status: ImportStatus.ready,
            changed: 1,
          ),
        ],
      );

    Future<(ProviderContainer, FakeExchangeSource)> previewed() async {
      final fake = source();
      final container = _container(
        source: fake,
        picker: FakeTextFilePicker(
          const PickedText(name: 'marks.json', content: '{}'),
        ),
      );
      final controller = container.read(importControllerProvider.notifier);
      expect(await controller.choose(), isTrue);
      await controller.loading;
      return (container, fake);
    }

    test('a chosen file is previewed first and nothing is written', () async {
      final (container, fake) = await previewed();
      final state = container.read(importControllerProvider);
      expect(state, isA<ImportPreview>());
      final preview = state as ImportPreview;
      expect(preview.fileName, 'marks.json');
      expect(preview.report.readyCount, 1);
      expect(preview.canApply, isTrue);
      expect(fake.calls, ['import dryRun=true overwrite=false json']);
    });

    test('choosing nothing changes nothing', () async {
      final container = _container();
      final controller = container.read(importControllerProvider.notifier);
      expect(await controller.choose(), isFalse);
      expect(container.read(importControllerProvider), isA<ImportIdle>());
    });

    test('only JSON and CSV files are taken', () async {
      final container = _container(
        picker: FakeTextFilePicker(
          const PickedText(name: 'marks.txt', content: ''),
        ),
      );
      final controller = container.read(importControllerProvider.notifier);
      expect(await controller.choose(), isTrue);
      expect(container.read(importControllerProvider), isA<ImportFailed>());
    });

    test(
      'overwriting asks the server again and may make a conflict ready',
      () async {
        final (container, fake) = await previewed();
        final controller = container.read(importControllerProvider.notifier);
        controller.setOverwrite(overwrite: true);
        expect(
          (container.read(
            importControllerProvider,
          ) as ImportPreview).refreshing,
          isTrue,
        );
        await controller.loading;
        final preview =
            container.read(importControllerProvider) as ImportPreview;
        expect(preview.overwrite, isTrue);
        expect(preview.report.readyCount, 2);
        expect(fake.calls.last, 'import dryRun=true overwrite=true json');
      },
    );

    test(
      'applying writes with the chosen overwrite and reports the outcome',
      () async {
        final (container, fake) = await previewed();
        final controller = container.read(importControllerProvider.notifier);
        await controller.apply();
        final state = container.read(importControllerProvider);
        expect(state, isA<ImportDone>());
        expect((state as ImportDone).applied, 1);
        expect(state.leftAsIs, 1);
        expect(fake.calls.last, 'import dryRun=false overwrite=false json');
      },
    );

    test('with no ready recording there is nothing to apply', () async {
      final fake = FakeExchangeSource()
        ..report = const ImportReport(dryRun: true, recordings: [_conflict]);
      final container = _container(
        source: fake,
        picker: FakeTextFilePicker(
          const PickedText(name: 'marks.csv', content: ''),
        ),
      );
      final controller = container.read(importControllerProvider.notifier);
      await controller.choose();
      await controller.loading;
      await controller.apply();
      expect(container.read(importControllerProvider), isA<ImportPreview>());
      expect(fake.calls, ['import dryRun=true overwrite=false csv']);
    });

    test('a file the server cannot read is reported with its reason', () async {
      final fake = FakeExchangeSource()
        ..failure = const Refused(ServerReason('import_malformed'));
      final container = _container(
        source: fake,
        picker: FakeTextFilePicker(
          const PickedText(name: 'marks.json', content: '{oops'),
        ),
      );
      final controller = container.read(importControllerProvider.notifier);
      await controller.choose();
      await controller.loading;
      final state = container.read(importControllerProvider);
      expect(state, isA<ImportFailed>());
      expect((state as ImportFailed).message, contains('nicht lesen'));
    });

    test('closing the import forgets the file', () async {
      final (container, _) = await previewed();
      final controller = container.read(importControllerProvider.notifier);
      controller.dismiss();
      expect(container.read(importControllerProvider), isA<ImportIdle>());
      await controller.apply();
      expect(container.read(importControllerProvider), isA<ImportIdle>());
    });
  });

  test('the new refusal codes are worded in the interface language', () {
    for (final code in [
      'outside_recording',
      'name_too_long',
      'import_malformed',
      'import_format_unsupported',
      'export_format_unknown',
    ]) {
      expect(knownReasonText(ServerReason(code)), isNotNull, reason: code);
    }
    expect(
      serverReasonText(const ServerReason('csv_invalid', args: [4])),
      contains('4'),
    );
  });
}
