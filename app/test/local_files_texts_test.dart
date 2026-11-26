import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/library/library_texts.dart';
import 'package:veltrunox_app/features/library/upload_bar.dart';

void main() {
  for (final local in [false, true]) {
    testWidgets('the button says what happens to a file (local: $local)', (
      tester,
    ) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [localFilesProvider.overrideWithValue(local)],
          child: const MaterialApp(home: Scaffold(body: UploadBar())),
        ),
      );

      expect(find.text(LibraryTexts.upload(local: local)), findsOneWidget);
      expect(find.text(LibraryTexts.upload(local: !local)), findsNothing);
    });
  }
}
