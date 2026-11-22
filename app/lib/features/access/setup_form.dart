import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'access_card.dart';
import 'access_models.dart';
import 'access_texts.dart';
import 'failure_notice.dart';
import 'password_field.dart';

/// The first start: creates the admin of a server that has no user yet.
class SetupForm extends StatefulWidget {
  const SetupForm({
    required this.busy,
    required this.failure,
    required this.onSetUp,
    super.key,
  });

  final bool busy;
  final AccessFailure? failure;
  final void Function(String userName, String password) onSetUp;

  @override
  State<SetupForm> createState() => _SetupFormState();
}

class _SetupFormState extends State<SetupForm> {
  /// A server that is set up now has the standard rules; the server decides.
  static const _policy = PasswordPolicy.standard;

  final _userName = TextEditingController();
  final _password = TextEditingController();
  final _again = TextEditingController();
  bool _submitted = false;

  @override
  void dispose() {
    _userName.dispose();
    _password.dispose();
    _again.dispose();
    super.dispose();
  }

  String? get _nameError =>
      _submitted && _userName.text.trim().isEmpty ? AccessTexts.required : null;

  String? get _passwordError {
    if (!_submitted) return null;
    final problem = _policy.check(_password.text);
    return problem == null
        ? null
        : AccessTexts.passwordProblem(_policy, problem);
  }

  String? get _againError =>
      _submitted && _again.text != _password.text ? AccessTexts.mismatch : null;

  void _submit() {
    if (widget.busy) return;
    setState(() => _submitted = true);
    if (_nameError != null || _passwordError != null || _againError != null) {
      return;
    }
    widget.onSetUp(_userName.text, _password.text);
  }

  @override
  Widget build(BuildContext context) => AccessCard(
    title: AccessTexts.setupTitle,
    child: AutofillGroup(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(AccessTexts.setupHint),
          const SizedBox(height: AppSpace.s4),
          FailureNotice(failure: widget.failure),
          TextField(
            controller: _userName,
            autofillHints: const [AutofillHints.newUsername],
            textInputAction: TextInputAction.next,
            decoration: InputDecoration(
              labelText: AccessTexts.userName,
              errorText: _nameError,
            ),
          ),
          const SizedBox(height: AppSpace.s3),
          PasswordField(
            controller: _password,
            label: AccessTexts.password,
            error: _passwordError,
            textInputAction: TextInputAction.next,
          ),
          const SizedBox(height: AppSpace.s3),
          PasswordField(
            controller: _again,
            label: AccessTexts.passwordAgain,
            error: _againError,
            textInputAction: TextInputAction.done,
            onSubmitted: _submit,
          ),
          const SizedBox(height: AppSpace.s4),
          ActionButton(
            label: AccessTexts.setup,
            loading: widget.busy,
            onPressed: _submit,
          ),
        ],
      ),
    ),
  );
}
