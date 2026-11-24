import 'package:material_ui/material_ui.dart';

import '../../app_shell/brand_mark.dart';
import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/frame_decoration.dart';
import '../language/language_menu.dart';

/// The frame of the screens before the app: the full logo, then a panel with
/// the heading and the content, centred and no wider than a phone, on a faint
/// teal glow.
class AccessCard extends StatelessWidget {
  const AccessCard({required this.title, required this.child, super.key});

  final String title;
  final Widget child;

  static const _maxWidth = 400.0;

  /// A faint teal light from above, behind the logo.
  static final _glow = RadialGradient(
    center: const Alignment(0, -0.7),
    radius: 1.1,
    colors: [
      Color.alphaBlend(
        AppColors.accent.withValues(alpha: 0.12),
        AppColors.canvas,
      ),
      AppColors.canvas,
    ],
  );

  @override
  Widget build(BuildContext context) => Scaffold(
    body: DecoratedBox(
      decoration: BoxDecoration(gradient: _glow),
      child: SafeArea(
        child: Stack(
          children: [
            const Align(alignment: Alignment.topRight, child: LanguageMenu()),
            Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(AppSpace.s4),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: _maxWidth),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      const Center(child: BrandMark.full()),
                      const SizedBox(height: AppSpace.s6),
                      DecoratedBox(
                        decoration: frameDecoration(),
                        child: Padding(
                          padding: const EdgeInsets.all(AppSpace.s6),
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              Semantics(
                                header: true,
                                child: Text(
                                  title,
                                  style: Theme.of(context)
                                      .textTheme
                                      .headlineSmall,
                                ),
                              ),
                              const SizedBox(height: AppSpace.s4),
                              child,
                            ],
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}
