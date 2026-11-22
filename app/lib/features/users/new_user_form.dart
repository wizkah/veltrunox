import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
import '../access/failure_notice.dart';
import 'users_texts.dart';

/// Creates a user: name, first password and role. A refusal by the server
/// shows above the fields, where the person just acted.
class NewUserForm extends StatefulWidget {
  const NewUserForm({required this.onCreate, super.key});

  /// Resolves to the failure, or `null` once the user exists.
  final Future<AccessFailure?> Function(
    String userName,
    String password,
    UserRole role,
  )
  onCreate;

  @override
  State<NewUserForm> createState() => _NewUserFormState();
}

class _NewUserFormState extends State<NewUserForm> {
  static const _minPasswordLength = 10;

  final _userName = TextEditingController();
  final _password = TextEditingController();
  UserRole _role = UserRole.user;
  bool _submitted = false;
  bool _busy = false;
  AccessFailure? _failure;

  @override
  void dispose() {
    _userName.dispose();
    _password.dispose();
    super.dispose();
  }

  String? get _nameError =>
      _submitted && _userName.text.trim().isEmpty ? UsersTexts.required : null;

  String? get _passwordError =>
      _submitted && _password.text.length < _minPasswordLength
      ? UsersTexts.tooShort
      : null;

  Future<void> _submit() async {
    setState(() => _submitted = true);
    if (_nameError != null || _passwordError != null) return;
    setState(() {
      _busy = true;
      _failure = null;
    });
    final failure = await widget.onCreate(
      _userName.text,
      _password.text,
      _role,
    );
    if (!mounted) return;
    setState(() {
      _busy = false;
      _failure = failure;
      if (failure == null) {
        _userName.clear();
        _password.clear();
        _submitted = false;
      }
    });
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Text(UsersTexts.newUser, style: Theme.of(context).textTheme.titleSmall),
      const SizedBox(height: AppSpace.s2),
      FailureNotice(failure: _failure),
      TextField(
        controller: _userName,
        decoration: InputDecoration(
          labelText: UsersTexts.userName,
          errorText: _nameError,
        ),
      ),
      const SizedBox(height: AppSpace.s2),
      TextField(
        controller: _password,
        obscureText: true,
        autocorrect: false,
        enableSuggestions: false,
        decoration: InputDecoration(
          labelText: AccessTexts.password,
          errorText: _passwordError,
        ),
      ),
      const SizedBox(height: AppSpace.s2),
      DropdownButtonFormField<UserRole>(
        initialValue: _role,
        decoration: const InputDecoration(labelText: UsersTexts.role),
        items: const [
          DropdownMenuItem(
            value: UserRole.user,
            child: Text(UsersTexts.roleUser),
          ),
          DropdownMenuItem(
            value: UserRole.admin,
            child: Text(UsersTexts.roleAdmin),
          ),
        ],
        onChanged: (role) => setState(() => _role = role ?? _role),
      ),
      const SizedBox(height: AppSpace.s3),
      ActionButton(
        label: UsersTexts.create,
        icon: Icons.person_add_alt,
        loading: _busy,
        onPressed: _submit,
      ),
    ],
  );
}
