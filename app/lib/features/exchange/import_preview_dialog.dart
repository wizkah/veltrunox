import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/activity_status.dart';
import 'exchange_texts.dart';
import 'import_controller.dart';
import 'import_outcome_row.dart';

/// The preview of an import and its confirmation. Nothing is written before
/// "Apply"; the dialog stays until the person closes it.
class ImportPreviewDialog extends ConsumerWidget {
  const ImportPreviewDialog({required this.titleOf, super.key});

  /// The title of a recording, if it is known.
  final String? Function(int recordingId) titleOf;

  static Future<void> show(
    BuildContext context, {
    required String? Function(int recordingId) titleOf,
  }) => showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (_) => ImportPreviewDialog(titleOf: titleOf),
  );

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(importControllerProvider);
    final controller = ref.read(importControllerProvider.notifier);
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 640, maxHeight: 720),
        child: Padding(
          padding: const EdgeInsets.all(AppSpace.s4),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Semantics(
                header: true,
                child: Text(
                  ExchangeTexts.previewTitle,
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ),
              const SizedBox(height: AppSpace.s3),
              Flexible(child: _content(context, state, controller)),
              const SizedBox(height: AppSpace.s3),
              _actions(context, state, controller),
            ],
          ),
        ),
      ),
    );
  }

  Widget _content(
    BuildContext context,
    ImportState state,
    ImportController controller,
  ) => switch (state) {
    ImportIdle() => const SizedBox.shrink(),
    ImportReading() => ActivityStatus(
      state: ActivityState.loading,
      label: ExchangeTexts.reading,
    ),
    ImportApplying() => ActivityStatus(
      state: ActivityState.loading,
      label: ExchangeTexts.applying,
    ),
    ImportDone(:final applied, :final leftAsIs) => ActivityStatus(
      state: ActivityState.success,
      label: ExchangeTexts.done(applied, leftAsIs),
    ),
    ImportFailed(:final message) => ActivityStatus(
      state: ActivityState.error,
      label: message,
    ),
    ImportPreview() => _preview(context, state, controller),
  };

  Widget _preview(
    BuildContext context,
    ImportPreview preview,
    ImportController controller,
  ) {
    final report = preview.report;
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(ExchangeTexts.previewIntro(preview.fileName)),
          const SizedBox(height: AppSpace.s2),
          for (final outcome in report.recordings)
            ImportOutcomeRow(
              title:
                  titleOf(outcome.recordingId) ??
                  ExchangeTexts.recordingFallback(outcome.recordingId),
              outcome: outcome,
            ),
          if (report.readyCount == 0) Text(ExchangeTexts.nothingReady),
          if (report.leftAsIsCount > 0) Text(ExchangeTexts.leftAsIsNote),
          if (report.hasConflicts || preview.overwrite)
            SwitchListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(ExchangeTexts.overwrite),
              subtitle: Text(ExchangeTexts.overwriteHint),
              value: preview.overwrite,
              onChanged: preview.refreshing
                  ? null
                  : (value) => controller.setOverwrite(overwrite: value),
            ),
          if (preview.refreshing)
            Padding(
              padding: const EdgeInsets.only(top: AppSpace.s2),
              child: ActivityStatus(
                state: ActivityState.loading,
                label: ExchangeTexts.reading,
              ),
            ),
        ],
      ),
    );
  }

  Widget _actions(
    BuildContext context,
    ImportState state,
    ImportController controller,
  ) {
    final working = state is ImportReading || state is ImportApplying;
    final deciding = state is ImportPreview;
    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        ActionButton(
          label: deciding ? ExchangeTexts.cancel : ExchangeTexts.close,
          kind: ActionKind.secondary,
          onPressed: state is ImportApplying
              ? null
              : () => Navigator.of(context).pop(),
        ),
        if (deciding || working) ...[
          const SizedBox(width: AppSpace.s2),
          ActionButton(
            label: ExchangeTexts.apply,
            loading: state is ImportApplying,
            onPressed: deciding && state.canApply ? controller.apply : null,
          ),
        ],
      ],
    );
  }
}
