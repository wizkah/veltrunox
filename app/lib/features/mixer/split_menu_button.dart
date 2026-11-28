import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import 'mixer_texts.dart';

/// "Split" with the choice of how many stems. Without a callback it is disabled.
class SplitMenuButton extends StatelessWidget {
  const SplitMenuButton({required this.onSplit, super.key});

  final ValueChanged<int>? onSplit;

  @override
  Widget build(BuildContext context) => MenuAnchor(
    menuChildren: [
      for (final (count, label) in [
        (4, MixerTexts.fourStems),
        (6, MixerTexts.sixStems),
      ])
        MenuItemButton(
          onPressed: onSplit == null ? null : () => onSplit!(count),
          child: Text(label),
        ),
    ],
    builder: (context, menu, _) => ActionButton(
      label: MixerTexts.split,
      icon: Icons.call_split,
      kind: ActionKind.secondary,
      onPressed: onSplit == null
          ? null
          : () => menu.isOpen ? menu.close() : menu.open(),
    ),
  );
}
