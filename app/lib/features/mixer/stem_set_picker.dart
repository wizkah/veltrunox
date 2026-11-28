import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/app_icon_button.dart';
import 'mixer_texts.dart';

/// The stem sets of a recording: which one plays, and removing it. A control
/// without a callback is disabled or not shown.
class StemSetPicker extends StatelessWidget {
  const StemSetPicker({
    required this.sets,
    required this.selected,
    this.onSelect,
    this.onDelete,
    super.key,
  });

  final List<String> sets;
  final String selected;
  final ValueChanged<String>? onSelect;
  final VoidCallback? onDelete;

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Flexible(
        child: DropdownButton<String>(
          style: Theme.of(context).textTheme.bodyMedium,
          dropdownColor: AppColors.surfaceRaised,
          value: selected,
          isExpanded: true,
          onChanged: onSelect == null
              ? null
              : (set) {
                  if (set != null && set != selected) onSelect!(set);
                },
          items: [
            for (final set in sets)
              DropdownMenuItem(
                value: set,
                child: Text(
                  MixerTexts.setName(set),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
          ],
        ),
      ),
      if (onDelete != null) ...[
        const SizedBox(width: AppSpace.s2),
        AppIconButton(
          icon: Icons.delete_outline,
          tooltip: MixerTexts.deleteSet,
          onPressed: onDelete,
        ),
      ],
    ],
  );
}
