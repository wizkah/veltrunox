import 'package:flutter/services.dart';

import '../features/tokens/token_source.dart';
import 'clipboard_fallback_stub.dart'
    if (dart.library.js_interop) 'clipboard_fallback_web.dart';

/// Adapter: the device's clipboard. In a browser without the clipboard API
/// (an insecure context) it falls back to the legacy copy command.
class SystemTextClipboard implements TextClipboard {
  @override
  Future<void> copy(String text) async {
    try {
      await Clipboard.setData(ClipboardData(text: text));
    } on Object {
      if (!copyTextWithFallback(text)) rethrow;
    }
  }
}
