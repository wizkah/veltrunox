import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/composition/mixer_board.dart';
import 'package:veltrunox_app/features/mixer/mixer_source.dart';
import 'package:veltrunox_app/features/mixer/stem_actions_bar.dart';
import 'package:veltrunox_app/features/mixer/stem_controls.dart';
import 'package:veltrunox_app/features/mixer/stem_models.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/waveform/playhead.dart';
import 'package:veltrunox_app/features/waveform/waveform_source.dart';

import 'fake_player_source.dart';

class _NoWaveforms implements MixerSource {
  @override
  Future<StemOverview> stems(int recordingId) async => const StemOverview();

  @override
  Future<void> split(int recordingId, SplitModel model) async {}

  @override
  Future<void> createRegions(int recordingId) async {}

  @override
  Future<LoadedWaveform?> waveform(int recordingId, int stem) async => null;

  @override
  Future<ImportedStems> importPackage(Uint8List package) =>
      throw UnimplementedError();

  @override
  Future<Uint8List> exportPackage(int recordingId) =>
      throw UnimplementedError();

  @override
  Future<void> remove(int recordingId) async {}
}

void main() {
  const stems = [Stem(name: 'drums', key: 'a'), Stem(name: 'bass', key: 'b')];

  Future<FakePlayerSource> show(
    WidgetTester tester, {
    required bool enabled,
    double width = 1200,
  }) async {
    final player = FakePlayerSource();
    tester.view.physicalSize = Size(width, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          playerSourceProvider.overrideWithValue(player),
          mixerSourceProvider.overrideWithValue(_NoWaveforms()),
        ],
        child: MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: MixerBoard(
                recordingId: 1,
                durationMs: 60000,
                stems: stems,
                connected: true,
                state: PlayerState(
                  recordingId: 1,
                  running: false,
                  speed: 1,
                  tuningHz: 440,
                  keyShiftSemitones: 0,
                  volumePercent: 50,
                  stemsEnabled: enabled,
                  stemChannels: const [
                    StemChannel(),
                    StemChannel(muted: true, audible: false),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 200));
    return player;
  }

  testWidgets('one lane per stem and one playhead through all', (tester) async {
    await show(tester, enabled: true);
    expect(find.byType(StemControls), findsNWidgets(2));
    expect(find.byType(Playhead), findsOneWidget);
    expect(find.text('drums'), findsOneWidget);
    expect(find.textContaining('bass · '), findsOneWidget);
  });

  testWidgets('solo and mute become commands', (tester) async {
    final player = await show(tester, enabled: true);
    await tester.tap(find.text('S').first);
    await tester.tap(find.text('M').last);
    expect(player.calls, ['stemSolo 0 true', 'stemMuted 1 false']);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('without the stem mix the controls take no input', (
    tester,
  ) async {
    final player = await show(tester, enabled: false);
    await tester.tap(find.text('S').first);
    expect(player.calls, isEmpty);
    expect(find.text('bass'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('splitting is offered until there are stems, no deleting', (
    tester,
  ) async {
    Future<void> bar({
      ValueChanged<SplitModel>? split,
      VoidCallback? regions,
    }) => tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: StemActionsBar(
            stemsEnabled: false,
            onSplit: split,
            onCreateRegions: regions,
          ),
        ),
      ),
    );
    final asked = <SplitModel>[];
    await bar(split: asked.add);
    expect(find.text('Aufteilen'), findsOneWidget);
    await tester.tap(find.text('Aufteilen'));
    await tester.pumpAndSettle();
    await tester.tap(find.textContaining('BS-RoFormer: 6 Segmente'));
    await tester.pumpAndSettle();
    expect(asked, [SplitModel.sw6]);
    expect(find.text('Regionen erstellen'), findsNothing);
    await bar(regions: () {});
    expect(find.text('Aufteilen'), findsNothing);
    expect(find.text('Regionen erstellen'), findsOneWidget);
    expect(find.text('Löschen'), findsNothing);
  });

  testWidgets('fits a phone', (tester) async {
    await show(tester, enabled: true, width: 360);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });
}
