import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'marker_controller.dart';
import 'marker_texts.dart';

/// For a narrow screen: a button that opens the marker list in a sheet. The
/// sheet shows what [panelBuilder] makes.
class MarkerSheetButton extends ConsumerWidget {
  const MarkerSheetButton({
    required this.recordingId,
    required this.panelBuilder,
    super.key,
  });

  final int recordingId;
  final WidgetBuilder panelBuilder;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final count = ref.watch(
      markersProvider(recordingId).select((markers) => markers.value?.length),
    );
    return OutlinedButton.icon(
      icon: const Icon(Icons.list),
      label: Text(
        count == null ? MarkerTexts.title : MarkerTexts.openMarkers(count),
      ),
      onPressed: () => showModalBottomSheet<void>(
        context: context,
        isScrollControlled: true,
        showDragHandle: true,
        builder: (sheet) => DraggableScrollableSheet(
          expand: false,
          initialChildSize: 0.8,
          maxChildSize: 0.95,
          builder: (context, scrollController) => SingleChildScrollView(
            controller: scrollController,
            padding: const EdgeInsets.fromLTRB(
              AppSpace.s4,
              0,
              AppSpace.s4,
              AppSpace.s6,
            ),
            child: panelBuilder(context),
          ),
        ),
      ),
    );
  }
}
