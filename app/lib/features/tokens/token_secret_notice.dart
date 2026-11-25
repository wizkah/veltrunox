import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'tokens_texts.dart';

/// The text of a new token, shown once: with a copy action and the plain
/// statement that it cannot be shown again. The text lives only in this
/// widget's input; hiding the notice removes it from the screen.
class TokenSecretNotice extends StatelessWidget {
  const TokenSecretNotice({
    required this.token,
    required this.copied,
    required this.onCopy,
    required this.onHide,
    super.key,
  });

  final String token;

  /// The text was put on the clipboard.
  final bool copied;
  final VoidCallback onCopy;
  final VoidCallback onHide;

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: DecoratedBox(
      decoration: BoxDecoration(
        border: Border.all(color: AppColors.warning),
        borderRadius: BorderRadius.circular(AppSpace.s2),
      ),
      child: Padding(
        padding: const EdgeInsets.all(AppSpace.s3),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              TokensTexts.secretTitle,
              style: Theme.of(context).textTheme.titleSmall,
            ),
            const SizedBox(height: AppSpace.s1),
            Text(TokensTexts.secretWarning),
            const SizedBox(height: AppSpace.s2),
            SelectableText(
              token,
              style: const TextStyle(fontFamily: 'monospace'),
            ),
            const SizedBox(height: AppSpace.s2),
            Wrap(
              spacing: AppSpace.s2,
              runSpacing: AppSpace.s2,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                ActionButton(
                  label: TokensTexts.copy,
                  icon: Icons.copy,
                  onPressed: onCopy,
                ),
                if (copied) Text(TokensTexts.copied),
                ActionButton(
                  label: TokensTexts.hideSecret,
                  kind: ActionKind.secondary,
                  onPressed: onHide,
                ),
              ],
            ),
          ],
        ),
      ),
    ),
  );
}
