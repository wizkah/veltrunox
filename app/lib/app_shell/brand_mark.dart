import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_geometry.dart';
import 'shell_texts.dart';

/// The logo: the turtle mark and the wordmark, in their own proportions. It
/// shrinks as a whole where there is little room.
class BrandMark extends StatelessWidget {
  const BrandMark({super.key});

  static const _markHeight = 32.0;
  static const _wordmarkHeight = 16.0;

  @override
  Widget build(BuildContext context) => Semantics(
    label: ShellTexts.appName,
    image: true,
    child: ExcludeSemantics(
      child: FittedBox(
        fit: BoxFit.scaleDown,
        alignment: Alignment.centerLeft,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Image.asset('assets/logo/veltrunox-mark.png', height: _markHeight),
            const SizedBox(width: AppSpace.s2),
            Image.asset(
              'assets/logo/veltrunox-wordmark.png',
              height: _wordmarkHeight,
            ),
          ],
        ),
      ),
    ),
  );
}
