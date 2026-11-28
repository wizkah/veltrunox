import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import 'mixer_texts.dart';
import 'stem_models.dart';

/// "Split" with the choice of the model. Without a callback it is disabled.
class SplitMenuButton extends StatelessWidget {
  const SplitMenuButton({
    required this.onSplit,
    this.existing = const {},
    super.key,
  });

  final ValueChanged<SplitModel>? onSplit;

  /// Models whose stems are there already; they are not offered until deleted.
  final Set<String> existing;

  @override
  Widget build(BuildContext context) => MenuAnchor(
    menuChildren: [
      for (final model in SplitModel.values)
        MenuItemButton(
          onPressed: onSplit == null || existing.contains(model.name)
              ? null
              : () => onSplit!(model),
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
