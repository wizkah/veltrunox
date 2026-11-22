import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../shared_ui/confirm_dialog.dart';
import '../../shared_ui/server_reason.dart';
import '../../shared_ui/toast_notice.dart';
import 'library_controller.dart';
import 'library_query_controller.dart';
import 'library_texts.dart';
import 'recording_source.dart';
import 'recording_summary.dart';
import 'rename_dialog.dart';

/// Asks for a new title and renames; a refusal is shown, not thrown.
Future<void> renameRecordingFlow(
  BuildContext context,
  WidgetRef ref,
  RecordingSummary recording,
) async {
  final title = await RenameDialog.ask(context, recording.title);
  if (title == null || title == recording.title || !context.mounted) return;
  await _guarded(
    context,
    () => ref.read(libraryProvider.notifier).rename(recording.id, title),
  );
}

/// Deletes after a confirmation that names the recording.
Future<void> deleteRecordingFlow(
  BuildContext context,
  WidgetRef ref,
  RecordingSummary recording,
) async {
  final confirmed = await ConfirmDialog.ask(
    context,
    title: LibraryTexts.deleteTitle,
    message: LibraryTexts.deleteMessage(recording.title),
    confirmLabel: LibraryTexts.delete,
    cancelLabel: LibraryTexts.cancel,
  );
  if (!confirmed || !context.mounted) return;
  final deleted = await _guarded(
    context,
    () => ref.read(libraryProvider.notifier).delete(recording.id),
  );
  if (deleted) ref.read(libraryDetailProvider.notifier).close();
}

/// Runs the action; `false` after showing why it failed.
Future<bool> _guarded(
  BuildContext context,
  Future<void> Function() action,
) async {
  String? reason;
  try {
    await action();
    return true;
  } on RecordingActionException catch (error) {
    reason = serverReasonText(error.message);
  } on Object catch (error) {
    reason = '$error';
  }
  if (context.mounted) _fail(context, reason);
  return false;
}

void _fail(BuildContext context, String reason) {
  ToastNotice.show(
    context,
    kind: ToastKind.error,
    message: '${LibraryTexts.actionFailed}: $reason',
  );
}
