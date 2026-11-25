import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'library_query.dart';
import 'library_texts.dart';

/// Search and sort order of the library.
class LibraryToolbar extends StatelessWidget {
  const LibraryToolbar({
    required this.query,
    required this.onSearch,
    required this.onSort,
    super.key,
  });

  final LibraryQuery query;
  final ValueChanged<String> onSearch;
  final ValueChanged<LibrarySort> onSort;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Row(
        children: [
          Expanded(
            child: TextField(
              onChanged: onSearch,
              decoration: InputDecoration(
                hintText: LibraryTexts.search,
                prefixIcon: Icon(Icons.search),
              ),
              textInputAction: TextInputAction.search,
            ),
          ),
          const SizedBox(width: AppSpace.s2),
          _SortMenu(selected: query.sort, onSelected: onSort),
        ],
      ),
    ],
  );
}

class _SortMenu extends StatelessWidget {
  const _SortMenu({required this.selected, required this.onSelected});

  final LibrarySort selected;
  final ValueChanged<LibrarySort> onSelected;

  @override
  Widget build(BuildContext context) => PopupMenuButton<LibrarySort>(
    tooltip: LibraryTexts.sort,
    initialValue: selected,
    onSelected: onSelected,
    itemBuilder: (_) => [
      for (final sort in LibrarySort.values)
        CheckedPopupMenuItem(
          value: sort,
          checked: sort == selected,
          child: Text(_label(sort)),
        ),
    ],
    child: Padding(
      padding: const EdgeInsets.all(AppSpace.s2),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.sort),
          // A phone has no room for the words; the icon and tooltip remain.
          if (MediaQuery.sizeOf(context).width >= AppBreakpoint.tabletMin) ...[
            const SizedBox(width: AppSpace.s1),
            Text('${LibraryTexts.sort}: ${_label(selected)}'),
          ],
        ],
      ),
    ),
  );

  static String _label(LibrarySort sort) => switch (sort) {
    LibrarySort.title => LibraryTexts.sortTitle,
    LibrarySort.tempo => LibraryTexts.sortTempo,
    LibrarySort.key => LibraryTexts.sortKey,
  };
}
