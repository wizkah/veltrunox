import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';

/// When this app was built, as set by the build script; empty in other builds.
const _builtAt = String.fromEnvironment('BUILD_TIME');

/// The build time in the title bar, to tell which build a browser shows.
class BuildStamp extends StatelessWidget {
  const BuildStamp({super.key});

  @override
  Widget build(BuildContext context) => _builtAt.isEmpty
      ? const SizedBox.shrink()
      : Padding(
          padding: const EdgeInsets.only(right: AppSpace.s2),
          child: Text(
            'Build $_builtAt',
            style: Theme.of(context).textTheme.labelSmall
                ?.copyWith(color: AppColors.textPrimary.withValues(alpha: 0.6)),
          ),
        );
}
