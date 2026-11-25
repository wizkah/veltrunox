import 'package:material_ui/material_ui.dart';

import '../../shared_ui/confirm_dialog.dart';
import '../../shared_ui/toast_notice.dart';
import 'marker.dart';
import 'marker_controller.dart';
import 'marker_texts.dart';
import 'marker_form_dialog.dart';

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

  /// Opens the form, with the start at `playheadMs` and the end at the end of
  /// the recording, and adds what it describes (a label if the end is left empty).
  Future<void> addRegion({required int playheadMs}) async {
    final draft = await MarkerFormDialog.show(
      context,
      heading: MarkerTexts.addRegion,
      startMs: playheadMs,
      endMs: lengthMs ?? playheadMs,
      lengthMs: lengthMs,
    );
    if (draft == null) return;
    _report(
      await controller.add(
        title: draft.title,
        startMs: draft.startMs,
        endMs: draft.endMs,
      ),
    );
  }

  /// Opens the form with the marker's values; the changed marker may turn from
  /// a label into a region or back.
  Future<void> edit(Marker marker) async {
    final draft = await MarkerFormDialog.show(
      context,
      heading: MarkerTexts.editMarker,
      title: marker.title,
      startMs: marker.startMs,
      endMs: switch (marker) {
        LabelMarker() => null,
        RegionMarker(:final endMs) => endMs,
      },
      lengthMs: lengthMs,
    );
    if (draft == null) return;
    _report(
      await controller.change(
        marker,
        title: draft.title,
        startMs: draft.startMs,
        endMs: draft.endMs,
      ),
    );
  }

  /// Asks for the name and times of one region covering [regions] (at least
  /// two) and replaces them by it. Returns whether it was written.
  Future<bool> combine(List<RegionMarker> regions) async {
    final span = spanOfRegions(regions);
    final draft = await MarkerFormDialog.show(
      context,
      heading: MarkerTexts.combineRegions,
      title: regions.first.title,
      startMs: span.startMs,
      endMs: span.endMs,
      lengthMs: lengthMs,
    );
    if (draft == null) return false;
    final accepted = await controller.combineRegions(
      regions,
      title: draft.title,
      startMs: draft.startMs,
      endMs: draft.endMs ?? span.endMs,
    );
    _report(accepted);
    return accepted;
  }

  /// Asks for the name and times of a new region covering [regions] and adds
  /// it; the regions stay. Returns whether it was written.
  Future<bool> addCovering(
    List<RegionMarker> regions, {
    required String title,
  }) async {
    final span = spanOfRegions(regions);
    final draft = await MarkerFormDialog.show(
      context,
      heading: MarkerTexts.regionFromSelection,
      title: title,
      startMs: span.startMs,
      endMs: span.endMs,
      lengthMs: lengthMs,
    );
    if (draft == null) return false;
    final accepted = await controller.add(
      title: draft.title,
      startMs: draft.startMs,
      endMs: draft.endMs,
    );
    _report(accepted);
    return accepted;
  }

  /// Adds a second region with the same times.
  Future<void> duplicate(RegionMarker region) async => _report(
    await controller.addRegion(
      title: MarkerTexts.copyName(region.title),
      startMs: region.startMs,
      endMs: region.endMs,
    ),
  );

  /// Replaces [region] by the parts before and after [positionMs].
  Future<void> split(RegionMarker region, int positionMs) async {
    if (positionMs <= region.startMs || positionMs >= region.endMs) {
      ToastNotice.show(
        context,
        kind: ToastKind.error,
        message: MarkerTexts.playheadOutsideRegion,
      );
      return;
    }
    _report(await controller.splitRegion(region, positionMs: positionMs));
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
      message: controller.failureMessage,
    );
  }
}
