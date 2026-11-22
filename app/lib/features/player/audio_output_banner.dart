import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'player_controller.dart';
import 'player_texts.dart';

/// Says that the server has no audio device, so nothing is heard. Nothing
/// while there is one or the server has not told yet.
class AudioOutputBanner extends ConsumerWidget {
  const AudioOutputBanner({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (ref.watch(audioOutputProvider).value != false) {
      return const SizedBox.shrink();
    }
    return Semantics(
      liveRegion: true,
      child: Material(
        color: AppColors.surfaceRaised,
        shape: const Border(
          bottom: BorderSide(
            color: AppColors.error,
            width: AppSize.borderWidth,
          ),
        ),
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSpace.s4,
            vertical: AppSpace.s2,
          ),
          child: Row(
            children: [
              Icon(Icons.volume_off, color: AppColors.error),
              SizedBox(width: AppSpace.s3),
              Expanded(child: Text(PlayerTexts.noAudioDevice)),
            ],
          ),
        ),
      ),
    );
  }
}
