import 'package:material_ui/material_ui.dart';

import '../../shared_ui/confirm_dialog.dart';
import '../../shared_ui/toast_notice.dart';
import 'label_form_dialog.dart';
import 'marker.dart';
import 'marker_controller.dart';
import 'marker_texts.dart';
import 'region_form_dialog.dart';

/// The user flows around markers: ask in a form or dialog, write, and tell the
/// user when the server did not take the change.
class MarkerFlows {
  MarkerFlows({
    required this.context,
    required this.controller,
    required this.lengthMs,
  });

  final BuildContext context;
  final MarkerController controller;

  /// The recording's length, if known.
  final int? lengthMs;

  /// Opens the region form, with the start at `playheadMs`, and adds the region.
  Future<void> addRegion({required int playheadMs}) async {
    final draft = await RegionFormDialog.show(
      context,
      heading: MarkerTexts.addRegion,
      startMs: playheadMs,
      endMs: lengthMs ?? playheadMs,
      lengthMs: lengthMs,
    );
    if (draft == null) return;
    _report(
      await controller.addRegion(
        title: draft.title,
        startMs: draft.startMs,
        endMs: draft.endMs,
      ),
    );
  }

  Future<void> edit(Marker marker) => switch (marker) {
    final LabelMarker label => _editLabel(label),
    final RegionMarker region => _editRegion(region),
  };

  Future<void> _editLabel(LabelMarker label) async {
    final draft = await LabelFormDialog.show(
      context,
      title: label.title,
      positionMs: label.positionMs,
      lengthMs: lengthMs,
    );
    if (draft == null) return;
    _report(
      await controller.updateLabel(
        label,
        title: draft.title,
        positionMs: draft.positionMs,
      ),
    );
  }

  Future<void> _editRegion(RegionMarker region) async {
    final draft = await RegionFormDialog.show(
      context,
      heading: MarkerTexts.editRegion,
      title: region.title,
      startMs: region.startMs,
      endMs: region.endMs,
      lengthMs: lengthMs,
    );
    if (draft == null) return;
    _report(
      await controller.updateRegion(
        region,
        title: draft.title,
        startMs: draft.startMs,
        endMs: draft.endMs,
      ),
    );
  }

  Future<void> delete(Marker marker) async {
    final confirmed = await ConfirmDialog.ask(
      context,
      title: MarkerTexts.deleteTitle,
      message: MarkerTexts.deleteMessage(marker.title),
      confirmLabel: MarkerTexts.delete,
      cancelLabel: MarkerTexts.cancel,
    );
    if (confirmed) _report(await controller.delete(marker));
  }

  void _report(bool accepted) {
    if (accepted || !context.mounted) return;
    ToastNotice.show(
      context,
      kind: ToastKind.error,
      message: MarkerTexts.saveFailed,
    );
  }
}
