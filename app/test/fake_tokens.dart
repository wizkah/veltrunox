import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/tokens/token_models.dart';
import 'package:veltrunox_app/features/tokens/token_source.dart';

/// The API tokens of a person, in memory; a test can make calls fail.
class FakeTokenSource implements TokenSource {
  FakeTokenSource([List<ApiToken> tokens = const []]) : tokens = [...tokens];

  final List<ApiToken> tokens;
  final calls = <String>[];
  AccessFailure? failure;
  var _nextId = 100;

  void _maybeFail() {
    final error = failure;
    if (error != null) throw AccessException(error);
  }

  @override
  Future<List<ApiToken>> list() async {
    calls.add('list');
    return List.of(tokens);
  }

  @override
  Future<CreatedToken> create({
    required String name,
    required TokenScope scope,
  }) async {
    calls.add('create $name ${scope.name}');
    _maybeFail();
    final token = ApiToken(
      id: _nextId++,
      name: name,
      scope: scope,
      createdAt: 1700000000,
    );
    tokens.add(token);
    return CreatedToken(token: 'vxt_secret${token.id}', apiToken: token);
  }

  @override
  Future<void> revoke(int id) async {
    calls.add('revoke $id');
    _maybeFail();
    tokens.removeWhere((token) => token.id == id);
  }
}

class FakeClipboard implements TextClipboard {
  final copied = <String>[];

  @override
  Future<void> copy(String text) async => copied.add(text);
}
