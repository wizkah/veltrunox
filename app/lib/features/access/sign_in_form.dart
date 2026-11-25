import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'access_card.dart';
import 'access_models.dart';
import 'access_texts.dart';
import 'failure_notice.dart';
import 'password_field.dart';

/// Signing in with user name and password, and as a guest where the server
/// lets guests in.
class SignInForm extends StatefulWidget {
  const SignInForm({
    required this.guestEnabled,
    required this.busy,
    this.passwordChanged = false,
    required this.failure,
    required this.onSignIn,
    required this.onGuest,
    super.key,
  });

  final bool guestEnabled;
  final bool busy;
  final AccessFailure? failure;

  /// Says that the password was changed and the new one is needed now.
  final bool passwordChanged;
  final void Function(String userName, String password) onSignIn;
  final VoidCallback onGuest;

  @override
  State<SignInForm> createState() => _SignInFormState();
}

class _SignInFormState extends State<SignInForm> {
  final _userName = TextEditingController();
  final _password = TextEditingController();

  @override
  void dispose() {
    _userName.dispose();
    _password.dispose();
    super.dispose();
  }

  void _submit() {
    if (widget.busy) return;
    widget.onSignIn(_userName.text, _password.text);
  }

  @override
  Widget build(BuildContext context) => AccessCard(
    title: AccessTexts.signInTitle,
    child: AutofillGroup(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (widget.passwordChanged) const _PasswordChangedNotice(),
          FailureNotice(failure: widget.failure),
          TextField(
            controller: _userName,
            autofillHints: const [AutofillHints.username],
            textInputAction: TextInputAction.next,
            decoration: InputDecoration(labelText: AccessTexts.userName),
          ),
          const SizedBox(height: AppSpace.s3),
          PasswordField(
            controller: _password,
            label: AccessTexts.password,
            textInputAction: TextInputAction.done,
            onSubmitted: _submit,
          ),
          const SizedBox(height: AppSpace.s4),
          ActionButton(
            label: AccessTexts.signIn,
            loading: widget.busy,
            onPressed: _submit,
          ),
          if (widget.guestEnabled) ...[
            const SizedBox(height: AppSpace.s2),
            ActionButton(
              label: AccessTexts.guest,
              kind: ActionKind.secondary,
              onPressed: widget.busy ? null : widget.onGuest,
            ),
          ],
        ],
      ),
    ),
  );
}

class _PasswordChangedNotice extends StatelessWidget {
  const _PasswordChangedNotice();

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: Padding(
      padding: const EdgeInsets.only(bottom: AppSpace.s3),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.check_circle_outline),
          const SizedBox(width: AppSpace.s2),
          Expanded(child: Text(AccessTexts.passwordChangedSignIn)),
        ],
      ),
    ),
  );
}
