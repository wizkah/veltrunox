import 'package:flutter/services.dart';

import '../features/tokens/token_source.dart';

/// Adapter: the device's clipboard.
class SystemTextClipboard implements TextClipboard {
  @override
  Future<void> copy(String text) =>
      Clipboard.setData(ClipboardData(text: text));
}
