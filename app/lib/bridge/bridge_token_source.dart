import '../features/tokens/token_models.dart';
import '../features/tokens/token_source.dart';
import 'bridge_access_source.dart';
import 'generated/api.dart' as rust;

/// Adapter: API tokens through the Rust client core.
class BridgeTokenSource implements TokenSource {
  BridgeTokenSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<List<ApiToken>> list() async {
    final found = await guardAccess(
      () => rust.listApiTokens(serverUrl: serverUrl),
    );
    return [for (final token in found) _token(token)];
  }

  @override
  Future<CreatedToken> create({
    required String name,
    required TokenScope scope,
  }) async {
    final created = await guardAccess(
      () => rust.createApiToken(
        serverUrl: serverUrl,
        name: name,
        scope: switch (scope) {
          TokenScope.readOnly => rust.TokenScopeChoice.readOnly,
          TokenScope.editBookmarks => rust.TokenScopeChoice.editBookmarks,
        },
      ),
    );
    return CreatedToken(
      token: created.token,
      apiToken: _token(created.apiToken),
    );
  }

  @override
  Future<void> revoke(int id) => guardAccess(
    () => rust.revokeApiToken(serverUrl: serverUrl, tokenId: BigInt.from(id)),
  );

  static ApiToken _token(rust.ApiTokenItem token) => ApiToken(
    id: token.id.toInt(),
    name: token.name,
    scope: switch (token.scope) {
      rust.TokenScopeChoice.readOnly => TokenScope.readOnly,
      rust.TokenScopeChoice.editBookmarks => TokenScope.editBookmarks,
    },
    createdAt: token.createdAt.toInt(),
    lastUsed: token.lastUsed?.toInt(),
  );
}
