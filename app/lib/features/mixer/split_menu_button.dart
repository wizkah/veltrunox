import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import 'mixer_texts.dart';
import 'stem_models.dart';

/// "Split" with the choice of the model. Without a callback it is disabled.
class SplitMenuButton extends StatelessWidget {
  const SplitMenuButton({required this.onSplit, super.key});

  final ValueChanged<SplitModel>? onSplit;

  @override
  Widget build(BuildContext context) => MenuAnchor(
    menuChildren: [
      for (final model in SplitModel.values)
        MenuItemButton(
          onPressed: onSplit == null ? null : () => onSplit!(model),
          child: Text(MixerTexts.splitModel(model)),
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
