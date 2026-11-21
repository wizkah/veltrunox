import 'package:material_ui/material_ui.dart';

import 'features/library/library_screen.dart';

class VeltrunoxApp extends StatelessWidget {
  const VeltrunoxApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Veltrunox',
    theme: ThemeData(colorSchemeSeed: Colors.indigo),
    home: const LibraryScreen(),
  );
}
