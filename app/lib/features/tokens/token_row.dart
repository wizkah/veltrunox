import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'token_models.dart';
import 'tokens_texts.dart';

/// One token of the person: name, reach, when it was made and last used, and
/// the way to revoke it. The text of a token is never listed.
class TokenRow extends StatelessWidget {
  const TokenRow({required this.token, required this.onRevoke, super.key});

  final ApiToken token;
  final VoidCallback onRevoke;

  @override
  Widget build(BuildContext context) {
    final lastUsed = token.lastUsed;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: AppSpace.s2),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(token.name, style: Theme.of(context).textTheme.titleSmall),
                Text(TokensTexts.scope(token.scope)),
                Text(
                  '${TokensTexts.created(TokensTexts.moment(token.createdAt))}'
                  ' · ${TokensTexts.lastUsed(lastUsed == null ? null : TokensTexts.moment(lastUsed))}',
                  style: const TextStyle(color: AppColors.textSecondary),
                ),
              ],
            ),
          ),
          IconButton(
            tooltip: '${TokensTexts.revoke}: ${token.name}',
            icon: const Icon(Icons.delete_outline, color: AppColors.error),
            onPressed: onRevoke,
          ),
        ],
      ),
    );
  }
}
