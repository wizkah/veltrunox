import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_geometry.dart';
import 'shell_texts.dart';

/// The logo: the turtle mark and the wordmark, in their own proportions. It
/// shrinks as a whole where there is little room. The compact form stands in
/// a bar, mark beside wordmark; the full form greets on the screens before the
/// app, a large mark above the wordmark.
class BrandMark extends StatelessWidget {
  const BrandMark({super.key}) : full = false;

  const BrandMark.full({super.key}) : full = true;

  final bool full;

  static const _markHeight = 32.0;
  static const _wordmarkHeight = 16.0;
  static const _fullMarkHeight = 112.0;
  static const _fullWordmarkHeight = 28.0;

  static const _mark = 'assets/logo/veltrunox-mark.png';
  static const _wordmark = 'assets/logo/veltrunox-wordmark.png';

  @override
  Widget build(BuildContext context) => Semantics(
    label: ShellTexts.appName,
    image: true,
    child: ExcludeSemantics(
      child: FittedBox(
        fit: BoxFit.scaleDown,
        alignment: full ? Alignment.center : Alignment.centerLeft,
        child: full ? _stacked() : _inline(),
      ),
    ),
  );

  Widget _inline() => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Image.asset(_mark, height: _markHeight),
      const SizedBox(width: AppSpace.s2),
      Image.asset(_wordmark, height: _wordmarkHeight),
    ],
  );

  Widget _stacked() => Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      Image.asset(_mark, height: _fullMarkHeight),
      const SizedBox(height: AppSpace.s3),
      Image.asset(_wordmark, height: _fullWordmarkHeight),
    ],
  );
}
