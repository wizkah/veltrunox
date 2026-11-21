import 'dart:async';

import 'package:veltrunox_app/features/player/player_source.dart';
import 'package:veltrunox_app/features/player/player_state.dart';

/// Records the commands and lets a test publish player states.
class FakePlayerSource implements PlayerSource {
  final states = StreamController<PlayerState>.broadcast();
  final refusals = StreamController<String>.broadcast();
  final calls = <String>[];
  int? currentPosition = 0;

  @override
  Stream<PlayerState> watchState() => states.stream;

  @override
  Stream<String> rejections() => refusals.stream;

  @override
  Future<int?> position() async => currentPosition;

  @override
  Future<void> select(int recordingId) async =>
      calls.add('select $recordingId');

  @override
  Future<void> play() async => calls.add('play');

  @override
  Future<void> pause() async => calls.add('pause');

  @override
  Future<void> seek(int positionMs) async => calls.add('seek $positionMs');

  @override
  Future<void> setSpeed(double speed) async => calls.add('speed $speed');

  @override
  Future<void> setTuning(double hertz) async => calls.add('tuning $hertz');

  @override
  Future<void> setKey(int semitones) async => calls.add('key $semitones');
}
