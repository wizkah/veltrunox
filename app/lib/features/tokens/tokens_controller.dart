import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../access/access_models.dart';
import 'token_models.dart';
import 'token_source.dart';

/// What making a token came to: the token with its text, or why not.
class TokenCreation {
  const TokenCreation.created(CreatedToken this.token) : failure = null;

  const TokenCreation.failed(AccessFailure this.failure) : token = null;

  final CreatedToken? token;
  final AccessFailure? failure;
}

final tokensProvider =
    AsyncNotifierProvider.autoDispose<TokensController, List<ApiToken>>(
      TokensController.new,
    );

/// The API tokens of the person: each change goes to the server and the list
/// is read again, so it shows what the server holds. The text of a new token
/// goes to the caller and is not kept here.
class TokensController extends AsyncNotifier<List<ApiToken>> {
  TokenSource get _source => ref.read(tokenSourceProvider);

  @override
  Future<List<ApiToken>> build() => _source.list();

  Future<TokenCreation> create({
    required String name,
    required TokenScope scope,
  }) async {
    try {
      final created = await _source.create(name: name, scope: scope);
      await _reload();
      return TokenCreation.created(created);
    } on AccessException catch (error) {
      return TokenCreation.failed(error.failure);
    }
  }

  /// `null` once the token is gone, else why not.
  Future<AccessFailure?> revoke(int id) async {
    AccessFailure? failure;
    try {
      await _source.revoke(id);
    } on AccessException catch (error) {
      failure = error.failure;
    }
    await _reload();
    return failure;
  }

  Future<void> _reload() async {
    final refreshed = await AsyncValue.guard(_source.list);
    // A failed refresh keeps the list that is shown.
    if (ref.mounted && !refreshed.hasError) state = refreshed;
  }
}
