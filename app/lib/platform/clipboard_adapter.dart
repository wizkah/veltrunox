import 'package:flutter/services.dart';

import '../features/tokens/token_source.dart';
import 'clipboard_fallback_stub.dart'
    if (dart.library.js_interop) 'clipboard_fallback_web.dart';

/// Adapter: the device's clipboard. In a browser it first uses the legacy copy
/// command, which also works without the clipboard API (an insecure context).
class SystemTextClipboard implements TextClipboard {
  @override
  Future<void> copy(String text) async {
    // Before any await: browsers allow this only directly after a click.
    if (copyTextInBrowser(text)) return;
    await Clipboard.setData(ClipboardData(text: text));
  }
}
