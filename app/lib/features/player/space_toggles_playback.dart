import 'package:flutter/services.dart';
import 'package:material_ui/material_ui.dart';

/// While its page is on top, the space bar calls [onToggle] instead of
/// pressing the focused control; a focused text input keeps the key.
class SpaceTogglesPlayback extends StatefulWidget {
  const SpaceTogglesPlayback({
    required this.onToggle,
    required this.child,
    super.key,
  });

  /// `null` while playback cannot be controlled.
  final VoidCallback? onToggle;
  final Widget child;

  @override
  State<SpaceTogglesPlayback> createState() => _SpaceTogglesPlaybackState();
}

class _SpaceTogglesPlaybackState extends State<SpaceTogglesPlayback> {
  @override
  void initState() {
    super.initState();
    FocusManager.instance.addEarlyKeyEventHandler(_onKey);
  }

  @override
  void dispose() {
    FocusManager.instance.removeEarlyKeyEventHandler(_onKey);
    super.dispose();
  }

  KeyEventResult _onKey(KeyEvent event) {
    final toggle = widget.onToggle;
    final keys = HardwareKeyboard.instance;
    if (toggle == null ||
        event.logicalKey != LogicalKeyboardKey.space ||
        keys.isControlPressed ||
        keys.isMetaPressed ||
        keys.isAltPressed ||
        // A dialog, menu or sheet over the page has the keyboard.
        !(ModalRoute.of(context)?.isCurrent ?? true) ||
        _typing) {
      return KeyEventResult.ignored;
    }
    // Holding the key toggles once.
    if (event is KeyDownEvent) toggle();
    return KeyEventResult.handled;
  }

  bool get _typing {
    final focused = FocusManager.instance.primaryFocus?.context;
    return focused != null &&
        (focused.widget is EditableText ||
            focused.findAncestorWidgetOfExactType<EditableText>() != null);
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
