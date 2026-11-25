import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/activity_status.dart';
import '../../shared_ui/confirm_dialog.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
import 'new_token_form.dart';
import 'token_models.dart';
import 'token_row.dart';
import 'token_secret_notice.dart';
import 'token_source.dart';
import 'tokens_controller.dart';
import 'tokens_texts.dart';

/// The API tokens of the signed-in person: the list, a new token and, once
/// after making one, its text. Opened from the account menu; it is not a place
/// in the navigation.
class TokensDialog extends ConsumerStatefulWidget {
  const TokensDialog({super.key});

  static Future<void> show(BuildContext context) =>
      showDialog<void>(context: context, builder: (_) => const TokensDialog());

  @override
  ConsumerState<TokensDialog> createState() => _TokensDialogState();
}

class _TokensDialogState extends ConsumerState<TokensDialog> {
  /// The text of the token just made; gone when the notice is hidden or the
  /// dialog closes.
  String? _secret;
  bool _copied = false;

  Future<AccessFailure?> _create(String name, TokenScope scope) async {
    final outcome = await ref
        .read(tokensProvider.notifier)
        .create(name: name, scope: scope);
    final created = outcome.token;
    if (created != null && mounted) {
      setState(() {
        _secret = created.token;
        _copied = false;
      });
    }
    return outcome.failure;
  }

  Future<void> _copy() async {
    final secret = _secret;
    if (secret == null) return;
    await ref.read(textClipboardProvider).copy(secret);
    if (mounted) setState(() => _copied = true);
  }

  Future<void> _revoke(ApiToken token) async {
    final confirmed = await ConfirmDialog.ask(
      context,
      title: TokensTexts.revokeTitle(token.name),
      message: TokensTexts.revokeMessage,
      confirmLabel: TokensTexts.revoke,
      cancelLabel: TokensTexts.cancel,
    );
    if (!confirmed || !mounted) return;
    final failure = await ref.read(tokensProvider.notifier).revoke(token.id);
    if (failure == null || !mounted) return;
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(AccessTexts.failure(failure))));
  }

  @override
  Widget build(BuildContext context) => Dialog(
    child: ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 640, maxHeight: 720),
      // Its own messenger and scaffold keep messages inside the dialog.
      child: ScaffoldMessenger(
        child: Scaffold(
          backgroundColor: Colors.transparent,
          body: Padding(
            padding: const EdgeInsets.all(AppSpace.s4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Semantics(
                        header: true,
                        child: Text(
                          TokensTexts.title,
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                    ),
                    IconButton(
                      tooltip: TokensTexts.close,
                      icon: const Icon(Icons.close),
                      onPressed: () => Navigator.of(context).pop(),
                    ),
                  ],
                ),
                Flexible(
                  child: switch (ref.watch(tokensProvider)) {
                    AsyncData(:final value) => _content(value),
                    AsyncError() => _Failed(
                      onRetry: () => ref.invalidate(tokensProvider),
                    ),
                    _ => Center(
                      child: ActivityStatus(
                        state: ActivityState.loading,
                        label: TokensTexts.loading,
                      ),
                    ),
                  },
                ),
              ],
            ),
          ),
        ),
      ),
    ),
  );

  Widget _content(List<ApiToken> tokens) => ListView(
    shrinkWrap: true,
    children: [
      Text(TokensTexts.intro),
      const SizedBox(height: AppSpace.s3),
      if (_secret != null) ...[
        TokenSecretNotice(
          token: _secret!,
          copied: _copied,
          onCopy: _copy,
          onHide: () => setState(() => _secret = null),
        ),
        const SizedBox(height: AppSpace.s3),
      ],
      if (tokens.isEmpty) Text(TokensTexts.none),
      for (final token in tokens)
        TokenRow(token: token, onRevoke: () => _revoke(token)),
      const Divider(),
      NewTokenForm(onCreate: _create),
    ],
  );
}

class _Failed extends StatelessWidget {
  const _Failed({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      Text(TokensTexts.failure),
      const SizedBox(height: AppSpace.s3),
      ActionButton(label: TokensTexts.retry, onPressed: onRetry),
    ],
  );
}
