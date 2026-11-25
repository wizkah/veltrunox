import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../shared_ui/toast_notice.dart';
import 'exchange_models.dart';
import 'exchange_texts.dart';
import 'export_controller.dart';
import 'import_controller.dart';
import 'import_preview_dialog.dart';

enum _Choice { exportJson, exportCsv, import }

/// The entry to the bookmark exchange: export as JSON or CSV and, where the
/// person may edit bookmarks, import with a preview. [scope] says what an
/// export covers.
class ExchangeMenu extends ConsumerWidget {
  const ExchangeMenu({
    required this.scope,
    required this.canImport,
    required this.titleOf,
    super.key,
  });

  final ExportScope scope;
  final bool canImport;

  /// The title of a recording, for the import preview.
  final String? Function(int recordingId) titleOf;

  @override
  Widget build(BuildContext context, WidgetRef ref) => PopupMenuButton<_Choice>(
    tooltip: ExchangeTexts.menu,
    icon: const Icon(Icons.import_export),
    onSelected: (choice) => _run(context, ref, choice),
    itemBuilder: (_) => [
      PopupMenuItem(
        value: _Choice.exportJson,
        child: Text(ExchangeTexts.exportJson),
      ),
      PopupMenuItem(
        value: _Choice.exportCsv,
        child: Text(ExchangeTexts.exportCsv),
      ),
      if (canImport) ...[
        const PopupMenuDivider(),
        PopupMenuItem(value: _Choice.import, child: Text(ExchangeTexts.import)),
      ],
    ],
  );

  Future<void> _run(BuildContext context, WidgetRef ref, _Choice choice) async {
    switch (choice) {
      case _Choice.exportJson:
        await _export(context, ref, ExchangeFormat.json);
      case _Choice.exportCsv:
        await _export(context, ref, ExchangeFormat.csv);
      case _Choice.import:
        await _import(context, ref);
    }
  }

  Future<void> _export(
    BuildContext context,
    WidgetRef ref,
    ExchangeFormat format,
  ) async {
    final state = await ref
        .read(exportControllerProvider.notifier)
        .export(scope, format);
    final notice = exportNotice(state);
    if (notice == null || !context.mounted) return;
    ToastNotice.show(
      context,
      kind: state is ExportFailed ? ToastKind.error : ToastKind.success,
      message: notice,
    );
  }

  Future<void> _import(BuildContext context, WidgetRef ref) async {
    final controller = ref.read(importControllerProvider.notifier);
    if (!await controller.choose() || !context.mounted) return;
    await ImportPreviewDialog.show(context, titleOf: titleOf);
    controller.dismiss();
  }
}
