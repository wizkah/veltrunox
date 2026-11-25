import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../shared_ui/app_icon_button.dart';
import '../../shared_ui/clock_format.dart';
import 'marker.dart';
import 'marker_texts.dart';

enum _MarkerAction { edit, delete }

/// One marker in the list: type icon, name, time, a direct play button and a
/// menu with the other actions.
class MarkerRow extends StatelessWidget {
  const MarkerRow({
    required this.marker,
    required this.onPlay,
    required this.onEdit,
    required this.onDelete,
    this.active = false,
    this.selected,
    this.onToggleSelected,
    super.key,
  });

  final Marker marker;
  final VoidCallback onPlay;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  /// The region is the one being played.
  final bool active;

  /// Whether the row is ticked; `null` for a marker that cannot be ticked.
  final bool? selected;
  final VoidCallback? onToggleSelected;

  @override
  Widget build(BuildContext context) => ListTile(
    tileColor: active ? AppColors.action.withValues(alpha: 0.18) : null,
    shape: active
        ? const Border(left: BorderSide(color: AppColors.action, width: 4))
        : null,
    leading: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (selected != null)
          Semantics(
            label: MarkerTexts.select(marker.title),
            child: Checkbox(
              value: selected,
              onChanged: (_) => onToggleSelected?.call(),
            ),
          ),
        Icon(
          active
              ? Icons.graphic_eq
              : switch (marker) {
                  LabelMarker() => Icons.place_outlined,
                  RegionMarker() => Icons.linear_scale,
                },
          semanticLabel: active ? MarkerTexts.activeRegion : null,
          color: active ? AppColors.action : AppColors.accentSoft,
        ),
      ],
    ),
    title: Text(marker.title),
    subtitle: Text(_time),
    onTap: onPlay,
    trailing: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        AppIconButton(
          tooltip: MarkerTexts.play(marker.title),
          icon: Icons.play_arrow,
          onPressed: onPlay,
        ),
        PopupMenuButton<_MarkerAction>(
          tooltip: MarkerTexts.moreFor(marker.title),
          onSelected: (action) => switch (action) {
            _MarkerAction.edit => onEdit(),
            _MarkerAction.delete => onDelete(),
          },
          itemBuilder: (_) => [
            PopupMenuItem(
              value: _MarkerAction.edit,
              child: Text(MarkerTexts.edit),
            ),
            PopupMenuItem(
              value: _MarkerAction.delete,
              child: Text(MarkerTexts.delete),
            ),
          ],
        ),
      ],
    ),
  );

  String get _time => switch (marker) {
    final LabelMarker label => MarkerTexts.label(formatClock(label.positionMs)),
    final RegionMarker region => MarkerTexts.region(
      formatClock(region.startMs),
      formatClock(region.endMs),
    ),
  };
}
