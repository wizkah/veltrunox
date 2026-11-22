import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/player/remote_change_detector.dart';

PlayerState _state({
  int recording = 1,
  int position = 0,
  bool running = false,
  double speed = 1,
}) => PlayerState(
  recordingId: recording,
  running: running,
  speed: speed,
  tuningHz: 440,
  keyShiftSemitones: 0,
  volumePercent: 100,
  positionMs: position,
);

void main() {
  late DateTime clock;
  late RemoteChangeDetector detector;

  setUp(() {
    clock = DateTime(2026);
    detector = RemoteChangeDetector(now: () => clock);
  });

  void after(int milliseconds) =>
      clock = clock.add(Duration(milliseconds: milliseconds));

  test('the first state is not a change', () {
    expect(detector.isRemoteJump(_state(position: 5000)), isFalse);
  });

  test('a paused position that moved is a change by someone else', () {
    detector.isRemoteJump(_state(position: 5000));
    after(1000);
    expect(detector.isRemoteJump(_state(position: 72000)), isTrue);
  });

  test('ordinary playback is not a change', () {
    detector.isRemoteJump(_state(position: 5000, running: true));
    after(4000);
    expect(
      detector.isRemoteJump(_state(position: 9100, running: true)),
      isFalse,
    );
  });

  test('the speed is taken into account', () {
    detector.isRemoteJump(_state(position: 0, running: true, speed: 2));
    after(3000);
    expect(
      detector.isRemoteJump(_state(position: 6050, running: true, speed: 2)),
      isFalse,
    );
  });

  test('a change after an own action is the answer to it', () {
    detector.isRemoteJump(_state(position: 5000));
    detector.ownAction();
    after(300);
    expect(detector.isRemoteJump(_state(position: 72000)), isFalse);
  });

  test('another recording is no jump of the position', () {
    detector.isRemoteJump(_state(position: 5000));
    after(1000);
    expect(detector.isRemoteJump(_state(recording: 2, position: 0)), isFalse);
  });

  test('a stream of changes is reported once', () {
    detector.isRemoteJump(_state(position: 0));
    after(500);
    expect(detector.isRemoteJump(_state(position: 20000)), isTrue);
    after(200);
    expect(detector.isRemoteJump(_state(position: 30000)), isFalse);
    after(4000);
    expect(detector.isRemoteJump(_state(position: 50000)), isTrue);
  });
}
