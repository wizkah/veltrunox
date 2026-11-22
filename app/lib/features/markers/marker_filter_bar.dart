import 'package:material_ui/material_ui.dart';

import 'marker_order.dart';
import 'marker_texts.dart';

/// Chooses which kind of marker the list shows.
class MarkerFilterBar extends StatelessWidget {
  const MarkerFilterBar({
    required this.selected,
    required this.onSelected,
    super.key,
  });

  final MarkerFilter selected;
  final ValueChanged<MarkerFilter> onSelected;

  @override
  Widget build(BuildContext context) => SegmentedButton<MarkerFilter>(
    segments: [
      ButtonSegment(
        value: MarkerFilter.all,
        label: Text(MarkerTexts.filterAll),
      ),
      ButtonSegment(
        value: MarkerFilter.labels,
        label: Text(MarkerTexts.filterLabels),
      ),
      ButtonSegment(
        value: MarkerFilter.regions,
        label: Text(MarkerTexts.filterRegions),
      ),
    ],
    selected: {selected},
    showSelectedIcon: false,
    onSelectionChanged: (choice) => onSelected(choice.single),
  );
}
