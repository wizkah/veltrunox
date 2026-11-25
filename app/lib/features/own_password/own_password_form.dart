import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
import '../access/failure_notice.dart';
import '../access/password_field.dart';
import 'own_password_texts.dart';

/// The three fields for changing one's own password: the current one and the
/// new one twice. Problems the form can see are named at their field before
/// anything is sent; a refusal by the server shows where the person just
/// acted, and the fields keep what was typed.
class OwnPasswordForm extends StatefulWidget {
  const OwnPasswordForm({
    required this.policy,
    required this.onSubmit,
    required this.onCancel,
    super.key,
  });

  final PasswordPolicy policy;

  /// Resolves to the failure, or `null` once the password is changed.
  final Future<AccessFailure?> Function(String current, String next) onSubmit;
  final VoidCallback onCancel;

  @override
  State<OwnPasswordForm> createState() => _OwnPasswordFormState();
}

class _OwnPasswordFormState extends State<OwnPasswordForm> {
  final _current = TextEditingController();
  final _next = TextEditingController();
  final _repeat = TextEditingController();
  bool _submitted = false;
  bool _busy = false;
  AccessFailure? _failure;

  @override
  void dispose() {
    _current.dispose();
    _next.dispose();
    _repeat.dispose();
    super.dispose();
  }

  String? get _currentError {
    if (_submitted && _current.text.isEmpty) return OwnPasswordTexts.required;
    return _refusedAt(_Field.current);
  }

  String? get _nextError {
    if (_submitted) {
      final problem = widget.policy.check(_next.text);
      if (problem != null) {
        return AccessTexts.passwordProblem(widget.policy, problem);
      }
    }
    return _refusedAt(_Field.next);
  }

  String? get _repeatError => _submitted && _repeat.text != _next.text
      ? OwnPasswordTexts.mismatch
      : null;

  /// The server's refusal, worded at the field it concerns.
  String? _refusedAt(_Field field) {
    final failure = _failure;
    return failure != null && _fieldOf(failure) == field
        ? AccessTexts.failure(failure)
        : null;
  }

  Future<void> _submit() async {
    if (_busy) return;
    setState(() {
      _submitted = true;
      _failure = null;
    });
    if (_currentError != null || _nextError != null || _repeatError != null) {
      return;
    }
    setState(() => _busy = true);
    final failure = await widget.onSubmit(_current.text, _next.text);
    if (!mounted) return;
    setState(() {
      _busy = false;
      _failure = failure;
    });
  }

  @override
  Widget build(BuildContext context) {
    final failure = _failure;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // A failure that belongs to no field shows above them.
        FailureNotice(
          failure: failure != null && _fieldOf(failure) == null
              ? failure
              : null,
        ),
        PasswordField(
          controller: _current,
          label: OwnPasswordTexts.currentPassword,
          error: _currentError,
          textInputAction: TextInputAction.next,
        ),
        const SizedBox(height: AppSpace.s3),
        PasswordField(
          controller: _next,
          label: OwnPasswordTexts.newPassword,
          error: _nextError,
          helper: AccessTexts.passwordRule(widget.policy),
          textInputAction: TextInputAction.next,
        ),
        const SizedBox(height: AppSpace.s3),
        PasswordField(
          controller: _repeat,
          label: OwnPasswordTexts.repeatPassword,
          error: _repeatError,
          textInputAction: TextInputAction.done,
          onSubmitted: _submit,
        ),
        const SizedBox(height: AppSpace.s3),
        Text(
          OwnPasswordTexts.signOutNotice,
          style: Theme.of(context).textTheme.bodySmall,
        ),
        const SizedBox(height: AppSpace.s4),
        Wrap(
          alignment: WrapAlignment.end,
          spacing: AppSpace.s2,
          runSpacing: AppSpace.s2,
          children: [
            ActionButton(
              label: OwnPasswordTexts.cancel,
              kind: ActionKind.secondary,
              onPressed: _busy ? null : widget.onCancel,
            ),
            ActionButton(
              label: OwnPasswordTexts.save,
              loading: _busy,
              onPressed: _submit,
            ),
          ],
        ),
      ],
    );
  }
}

enum _Field { current, next }

/// The field a refusal is about, or `null` if it is about none.
_Field? _fieldOf(AccessFailure failure) => switch (failure) {
  Refused(:final reason) when reason.code == 'wrong_current_password' =>
    _Field.current,
  Refused(:final reason) when reason.code.startsWith('password_') =>
    _Field.next,
  _ => null,
};
