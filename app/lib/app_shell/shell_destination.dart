import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'shell_texts.dart';

/// The places the navigation leads to.
enum ShellDestination {
  player(ShellTexts.player, Icons.play_circle_outline, Icons.play_circle),
  library(
    ShellTexts.library,
    Icons.library_music_outlined,
    Icons.library_music,
  );

  const ShellDestination(this.label, this.icon, this.selectedIcon);

  final String label;
  final IconData icon;
  final IconData selectedIcon;
}

class ShellDestinationNotifier extends Notifier<ShellDestination> {
  @override
  ShellDestination build() => ShellDestination.library;

  void select(ShellDestination destination) => state = destination;
}

/// The destination currently shown; the library is where the app starts.
final shellDestinationProvider =
    NotifierProvider<ShellDestinationNotifier, ShellDestination>(
      ShellDestinationNotifier.new,
    );
