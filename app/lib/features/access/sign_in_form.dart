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
    required this.failure,
    required this.onSignIn,
    required this.onGuest,
    super.key,
  });

  final bool guestEnabled;
  final bool busy;
  final AccessFailure? failure;
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
