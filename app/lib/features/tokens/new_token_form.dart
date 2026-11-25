import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../access/access_models.dart';
import '../access/failure_notice.dart';
import 'token_models.dart';
import 'tokens_texts.dart';

/// Makes a token: a name and how far it reaches. A refusal of the server
/// shows above the fields, where the person just acted.
class NewTokenForm extends StatefulWidget {
  const NewTokenForm({required this.onCreate, super.key});

  /// Resolves to the failure, or `null` once the token is made.
  final Future<AccessFailure?> Function(String name, TokenScope scope) onCreate;

  @override
  State<NewTokenForm> createState() => _NewTokenFormState();
}

class _NewTokenFormState extends State<NewTokenForm> {
  final _name = TextEditingController();
  TokenScope _scope = TokenScope.editBookmarks;
  bool _submitted = false;
  bool _busy = false;
  AccessFailure? _failure;

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  String? get _nameError =>
      _submitted && _name.text.trim().isEmpty ? TokensTexts.required : null;

  Future<void> _submit() async {
    setState(() => _submitted = true);
    if (_nameError != null) return;
    setState(() {
      _busy = true;
      _failure = null;
    });
    final failure = await widget.onCreate(_name.text, _scope);
    if (!mounted) return;
    setState(() {
      _busy = false;
      _failure = failure;
      if (failure == null) {
        _name.clear();
        _submitted = false;
      }
    });
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Text(TokensTexts.newToken, style: Theme.of(context).textTheme.titleSmall),
      const SizedBox(height: AppSpace.s2),
      FailureNotice(failure: _failure),
      TextField(
        controller: _name,
        decoration: InputDecoration(
          labelText: TokensTexts.name,
          hintText: TokensTexts.nameHint,
          errorText: _nameError,
        ),
      ),
      const SizedBox(height: AppSpace.s2),
      DropdownButtonFormField<TokenScope>(
        initialValue: _scope,
        isExpanded: true,
        decoration: InputDecoration(
          labelText: TokensTexts.reach,
          helperText: TokensTexts.reachHint,
          helperMaxLines: 3,
        ),
        items: [
          for (final scope in TokenScope.values)
            DropdownMenuItem(
              value: scope,
              child: Text(TokensTexts.scope(scope)),
            ),
        ],
        onChanged: (scope) => setState(() => _scope = scope ?? _scope),
      ),
      const SizedBox(height: AppSpace.s3),
      Align(
        alignment: Alignment.centerLeft,
        child: ActionButton(
          label: TokensTexts.create,
          icon: Icons.add,
          loading: _busy,
          onPressed: _submit,
        ),
      ),
    ],
  );
}
