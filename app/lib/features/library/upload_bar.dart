import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/activity_status.dart';
import 'library_texts.dart';
import 'upload_controller.dart';
import 'upload_state.dart';

/// How many refused files the summary of a batch names.
const _failuresShown = 3;

/// The upload button with what the upload is doing: sending, done or failed.
class UploadBar extends ConsumerWidget {
  const UploadBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(uploadControllerProvider);
    final controller = ref.read(uploadControllerProvider.notifier);
    return Wrap(
      spacing: AppSpace.s3,
      runSpacing: AppSpace.s2,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        ActionButton(
          label: LibraryTexts.upload,
          icon: Icons.upload_file,
          onPressed: state is UploadSending ? null : controller.chooseAndUpload,
        ),
        ..._status(state, controller.dismiss),
      ],
    );
  }

  static List<Widget> _status(UploadState state, VoidCallback dismiss) {
    final status = switch (state) {
      UploadIdle() => null,
      UploadSending(
        :final fileName,
        :final position,
        :final total,
        :final percent,
      ) =>
        ActivityStatus(
          state: ActivityState.loading,
          label: LibraryTexts.uploading(
            fileName,
            position: position,
            total: total,
            percent: percent,
          ),
        ),
      UploadBatchDone(:final uploaded, :final existing, :final failures) =>
        ActivityStatus(
          state: failures.isEmpty ? ActivityState.success : ActivityState.error,
          label: [
            LibraryTexts.uploadSummary(
              uploaded: uploaded,
              existing: existing,
              failed: failures.length,
            ),
            for (final failure in failures.take(_failuresShown))
              LibraryTexts.uploadFailed(failure.fileName, failure.reason),
          ].join('\n'),
        ),
      UploadDone(:final title, :final existing) => ActivityStatus(
        state: ActivityState.success,
        label: existing
            ? LibraryTexts.uploadedAlready(title)
            : LibraryTexts.uploaded(title),
      ),
      UploadFailed(:final fileName, :final reason) => ActivityStatus(
        state: ActivityState.error,
        label: LibraryTexts.uploadFailed(fileName, reason),
      ),
    };
    if (status == null) return const [];
    return [
      status,
      if (state is! UploadSending)
        IconButton(
          tooltip: LibraryTexts.uploadDismiss,
          icon: const Icon(Icons.close),
          onPressed: dismiss,
        ),
    ];
  }
}
