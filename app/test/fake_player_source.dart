import 'dart:async';

import 'package:veltrunox_app/features/player/player_source.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/player/target_tempo.dart';

/// Records the commands and lets a test publish player states.
class FakePlayerSource implements PlayerSource {
  final states = StreamController<PlayerState>.broadcast();
  final refusals = StreamController<String>.broadcast();
  final calls = <String>[];
  int? currentPosition = 0;

  /// What `speedForTargetTempo` answers: the tempo ratio, supported within 0.5–2.
  bool failTargetTempo = false;

  @override
  Stream<PlayerState> watchState() => states.stream;

  @override
  Stream<String> rejections() => refusals.stream;

  final audioOutputs = StreamController<bool>.broadcast();

  @override
  Stream<bool> audioOutput() => audioOutputs.stream;

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
  Future<void> scrub(int positionMs) async => calls.add('scrub $positionMs');

  @override
  Future<void> endScrub(int positionMs) async =>
      calls.add('endScrub $positionMs');

  /// What `recordingLength` answers; `null` for a recording without a scan.
  int? length = 180000;

  final lengthRequests = <int>[];

  @override
  Future<int?> recordingLength(int recordingId) async {
    lengthRequests.add(recordingId);
    return length;
  }

  @override
  Future<void> setSpeed(double speed) async => calls.add('speed $speed');

  @override
  Future<void> setTuning(double hertz) async => calls.add('tuning $hertz');

  @override
  Future<void> setPlayMode(PlayMode mode) async =>
      calls.add('playMode ${mode.name}');

  @override
  Future<void> playRegion({required int startMs, required int endMs}) async =>
      calls.add('playRegion $startMs $endMs');

  @override
  Future<bool> tuningSupported(double hertz) async =>
      hertz >= 300 && hertz <= 500;

  @override
  Future<void> setKey(int semitones) async => calls.add('key $semitones');

  @override
  Future<void> setVolume(int percent) async => calls.add('volume $percent');

  @override
  Future<TargetTempo> speedForTargetTempo({
    required double targetBpm,
    required double recordingBpm,
  }) async {
    if (failTargetTempo) throw Exception('invalid tempo');
    final speed = targetBpm / recordingBpm;
    return TargetTempo(speed: speed, supported: speed >= 0.5 && speed <= 2);
  }
}
