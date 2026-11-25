import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'token_models.dart';

/// Port: the API tokens of the signed-in person. Failures are
/// `AccessException`s.
abstract interface class TokenSource {
  Future<List<ApiToken>> list();

  Future<CreatedToken> create({
    required String name,
    required TokenScope scope,
  });

  Future<void> revoke(int id);
}

/// Supplies the [TokenSource]; overridden at the application root.
final tokenSourceProvider = Provider<TokenSource>(
  (ref) => throw UnimplementedError('tokenSourceProvider must be overridden'),
);

/// Port: puts a text on the device's clipboard.
abstract interface class TextClipboard {
  Future<void> copy(String text);
}

/// Supplies the [TextClipboard]; overridden at the application root.
final textClipboardProvider = Provider<TextClipboard>(
  (ref) => throw UnimplementedError('textClipboardProvider must be overridden'),
);
