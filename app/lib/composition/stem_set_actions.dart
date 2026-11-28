import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../features/markers/marker_controller.dart';
import '../features/mixer/mixer_source.dart';
import '../features/mixer/stem_models.dart';
import '../features/mixer/mixer_texts.dart';
import '../shared_ui/confirm_dialog.dart';
import '../shared_ui/server_reason.dart';
import '../shared_ui/toast_notice.dart';

/// Whether a stem package is on its way.
class StemSetBusy extends Notifier<bool> {
  @override
  bool build() => false;

  void set(bool busy) => state = busy;
}

final stemSetBusyProvider = NotifierProvider<StemSetBusy, bool>(
  StemSetBusy.new,
);

/// Splitting, choosing and deleting stem sets, regions, import and export of a recording's stems, with the questions before
/// and the notices after. Used by the mixer page.
class StemSetActions {
  const StemSetActions(this.context, this.ref, this.recordingId);

  final BuildContext context;
  final WidgetRef ref;
  final int recordingId;

  MixerSource get _source => ref.read(mixerSourceProvider);

  Future<void> import({required bool replacing}) async {
    final package = await ref.read(stemPackageFilesProvider).pick();
    if (package == null || !context.mounted) return;
    if (replacing &&
        !await ConfirmDialog.ask(
          context,
          title: MixerTexts.replaceTitle,
          message: MixerTexts.replaceMessage,
          confirmLabel: MixerTexts.replace,
          cancelLabel: MixerTexts.cancel,
        )) {
      return;
    }
    await _run(() async {
      final imported = await _source.importPackage(package);
      return MixerTexts.imported(
        imported.title,
        imported.stems,
        imported.regionsAdded,
      );
    });
  }

  Future<void> split(SplitModel model) => _run(() async {
    await _source.split(recordingId, model);
    return null;
  });

  Future<void> select(String set) => _run(() async {
    await _source.select(recordingId, set);
    return null;
  });

  Future<void> delete(String set) async {
    if (!await ConfirmDialog.ask(
      context,
      title: MixerTexts.deleteTitle,
      message: MixerTexts.deleteMessage(MixerTexts.setName(set)),
      confirmLabel: MixerTexts.delete,
      cancelLabel: MixerTexts.cancel,
    )) {
      return;
    }
    await _run(() async {
      await _source.remove(recordingId, set);
      return null;
    });
  }

  Future<void> createRegions() => _run(() async {
    await _source.createRegions(recordingId);
    // Without a job they are there already; the mixer does not show them.
    final after = await _source.stems(recordingId);
    return after.analysingPercent == null && after.failureCode == null
        ? MixerTexts.regionsCreated
        : null;
  });

  Future<void> export(String title) => _run(() async {
    final package = await _source.exportPackage(recordingId);
    final saved = await ref
        .read(stemPackageFilesProvider)
        .save('$title.stems.zip', package);
    return saved ? MixerTexts.exported : null;
  });

  /// Runs the action, says what came of it and reads the stems again.
  Future<void> _run(Future<String?> Function() action) async {
    final busy = ref.read(stemSetBusyProvider.notifier);
    final container = ProviderScope.containerOf(context, listen: false);
    final messenger = ScaffoldMessenger.of(context);
    busy.set(true);
    String? message;
    var kind = ToastKind.success;
    try {
      message = await action();
    } on StemFailure catch (failure) {
      message = serverReasonText(failure.reason);
      kind = ToastKind.error;
    } finally {
      busy.set(false);
      container.invalidate(stemsProvider(recordingId));
      container.invalidate(markersProvider(recordingId));
    }
    if (message != null) {
      messenger.showSnackBar(
        SnackBar(
          content: ToastNotice(kind: kind, message: message),
        ),
      );
    }
  }
}
