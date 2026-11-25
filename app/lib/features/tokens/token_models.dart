/// How far an API token reaches, at most.
enum TokenScope {
  /// Reads library, bookmarks and exports.
  readOnly,

  /// Also edits, replaces and imports bookmarks.
  editBookmarks,
}

/// An API token as listed; its text is known only when it is made.
class ApiToken {
  const ApiToken({
    required this.id,
    required this.name,
    required this.scope,
    required this.createdAt,
    this.lastUsed,
  });

  final int id;
  final String name;
  final TokenScope scope;

  /// Unix seconds.
  final int createdAt;

  /// Unix seconds; `null` until the token was used.
  final int? lastUsed;
}

/// A new token with its text, which the server shows only this once.
class CreatedToken {
  const CreatedToken({required this.token, required this.apiToken});

  final String token;
  final ApiToken apiToken;
}
