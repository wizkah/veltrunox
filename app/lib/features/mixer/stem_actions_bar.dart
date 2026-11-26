import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'mixer_texts.dart';

/// The switch between stems and original, and what can be done with the stem
/// set. An action without a callback is not shown (missing right or no stems);
/// the switch needs stems.
class StemActionsBar extends StatelessWidget {
  const StemActionsBar({
    required this.stemsEnabled,
    this.onEnabled,
    this.onSplit,
    this.onCreateRegions,
    this.onImport,
    this.onExport,
    this.busy = false,
    this.zoom = const [],
    super.key,
  });

  final bool stemsEnabled;
  final ValueChanged<bool>? onEnabled;
  final VoidCallback? onSplit;
  final VoidCallback? onCreateRegions;
  final VoidCallback? onImport;
  final VoidCallback? onExport;

  /// A package is on its way; the actions wait.
  final bool busy;

  /// View buttons at the end of the bar.
  final List<Widget> zoom;

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: AppSpace.s2,
    runSpacing: AppSpace.s2,
    crossAxisAlignment: WrapCrossAlignment.center,
    children: [
      if (onEnabled != null)
        MergeSemantics(
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Switch(value: stemsEnabled, onChanged: onEnabled),
              const SizedBox(width: AppSpace.s2),
              Text(MixerTexts.stems),
            ],
          ),
        ),
      if (onSplit != null)
        ActionButton(
          label: MixerTexts.split,
          icon: Icons.call_split,
          kind: ActionKind.secondary,
          onPressed: busy ? null : onSplit,
        ),
      if (onCreateRegions != null)
        ActionButton(
          label: MixerTexts.createRegions,
          icon: Icons.auto_awesome,
          kind: ActionKind.secondary,
          onPressed: busy ? null : onCreateRegions,
        ),
      if (onImport != null)
        ActionButton(
          label: MixerTexts.import,
          icon: Icons.upload_file,
          kind: ActionKind.secondary,
          loading: busy,
          onPressed: onImport,
        ),
      if (onExport != null)
        ActionButton(
          label: MixerTexts.export,
          icon: Icons.download,
          kind: ActionKind.secondary,
          onPressed: busy ? null : onExport,
        ),
      ...zoom,
    ],
  );
}
