// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PlayerCommand {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayerCommand()';
}


}

/// @nodoc
class $PlayerCommandCopyWith<$Res>  {
$PlayerCommandCopyWith(PlayerCommand _, $Res Function(PlayerCommand) __);
}


/// Adds pattern-matching-related methods to [PlayerCommand].
extension PlayerCommandPatterns on PlayerCommand {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlayerCommand_SelectRecording value)?  selectRecording,TResult Function( PlayerCommand_Play value)?  play,TResult Function( PlayerCommand_Pause value)?  pause,TResult Function( PlayerCommand_Seek value)?  seek,TResult Function( PlayerCommand_SetSpeed value)?  setSpeed,TResult Function( PlayerCommand_SetTuning value)?  setTuning,TResult Function( PlayerCommand_SetKey value)?  setKey,TResult Function( PlayerCommand_SetVolume value)?  setVolume,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlayerCommand_SelectRecording() when selectRecording != null:
return selectRecording(_that);case PlayerCommand_Play() when play != null:
return play(_that);case PlayerCommand_Pause() when pause != null:
return pause(_that);case PlayerCommand_Seek() when seek != null:
return seek(_that);case PlayerCommand_SetSpeed() when setSpeed != null:
return setSpeed(_that);case PlayerCommand_SetTuning() when setTuning != null:
return setTuning(_that);case PlayerCommand_SetKey() when setKey != null:
return setKey(_that);case PlayerCommand_SetVolume() when setVolume != null:
return setVolume(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlayerCommand_SelectRecording value)  selectRecording,required TResult Function( PlayerCommand_Play value)  play,required TResult Function( PlayerCommand_Pause value)  pause,required TResult Function( PlayerCommand_Seek value)  seek,required TResult Function( PlayerCommand_SetSpeed value)  setSpeed,required TResult Function( PlayerCommand_SetTuning value)  setTuning,required TResult Function( PlayerCommand_SetKey value)  setKey,required TResult Function( PlayerCommand_SetVolume value)  setVolume,}){
final _that = this;
switch (_that) {
case PlayerCommand_SelectRecording():
return selectRecording(_that);case PlayerCommand_Play():
return play(_that);case PlayerCommand_Pause():
return pause(_that);case PlayerCommand_Seek():
return seek(_that);case PlayerCommand_SetSpeed():
return setSpeed(_that);case PlayerCommand_SetTuning():
return setTuning(_that);case PlayerCommand_SetKey():
return setKey(_that);case PlayerCommand_SetVolume():
return setVolume(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlayerCommand_SelectRecording value)?  selectRecording,TResult? Function( PlayerCommand_Play value)?  play,TResult? Function( PlayerCommand_Pause value)?  pause,TResult? Function( PlayerCommand_Seek value)?  seek,TResult? Function( PlayerCommand_SetSpeed value)?  setSpeed,TResult? Function( PlayerCommand_SetTuning value)?  setTuning,TResult? Function( PlayerCommand_SetKey value)?  setKey,TResult? Function( PlayerCommand_SetVolume value)?  setVolume,}){
final _that = this;
switch (_that) {
case PlayerCommand_SelectRecording() when selectRecording != null:
return selectRecording(_that);case PlayerCommand_Play() when play != null:
return play(_that);case PlayerCommand_Pause() when pause != null:
return pause(_that);case PlayerCommand_Seek() when seek != null:
return seek(_that);case PlayerCommand_SetSpeed() when setSpeed != null:
return setSpeed(_that);case PlayerCommand_SetTuning() when setTuning != null:
return setTuning(_that);case PlayerCommand_SetKey() when setKey != null:
return setKey(_that);case PlayerCommand_SetVolume() when setVolume != null:
return setVolume(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( BigInt recordingId)?  selectRecording,TResult Function()?  play,TResult Function()?  pause,TResult Function( BigInt positionMs)?  seek,TResult Function( double speed)?  setSpeed,TResult Function( double tuningHz)?  setTuning,TResult Function( int keyShiftSemitones)?  setKey,TResult Function( int volumePercent)?  setVolume,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlayerCommand_SelectRecording() when selectRecording != null:
return selectRecording(_that.recordingId);case PlayerCommand_Play() when play != null:
return play();case PlayerCommand_Pause() when pause != null:
return pause();case PlayerCommand_Seek() when seek != null:
return seek(_that.positionMs);case PlayerCommand_SetSpeed() when setSpeed != null:
return setSpeed(_that.speed);case PlayerCommand_SetTuning() when setTuning != null:
return setTuning(_that.tuningHz);case PlayerCommand_SetKey() when setKey != null:
return setKey(_that.keyShiftSemitones);case PlayerCommand_SetVolume() when setVolume != null:
return setVolume(_that.volumePercent);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( BigInt recordingId)  selectRecording,required TResult Function()  play,required TResult Function()  pause,required TResult Function( BigInt positionMs)  seek,required TResult Function( double speed)  setSpeed,required TResult Function( double tuningHz)  setTuning,required TResult Function( int keyShiftSemitones)  setKey,required TResult Function( int volumePercent)  setVolume,}) {final _that = this;
switch (_that) {
case PlayerCommand_SelectRecording():
return selectRecording(_that.recordingId);case PlayerCommand_Play():
return play();case PlayerCommand_Pause():
return pause();case PlayerCommand_Seek():
return seek(_that.positionMs);case PlayerCommand_SetSpeed():
return setSpeed(_that.speed);case PlayerCommand_SetTuning():
return setTuning(_that.tuningHz);case PlayerCommand_SetKey():
return setKey(_that.keyShiftSemitones);case PlayerCommand_SetVolume():
return setVolume(_that.volumePercent);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( BigInt recordingId)?  selectRecording,TResult? Function()?  play,TResult? Function()?  pause,TResult? Function( BigInt positionMs)?  seek,TResult? Function( double speed)?  setSpeed,TResult? Function( double tuningHz)?  setTuning,TResult? Function( int keyShiftSemitones)?  setKey,TResult? Function( int volumePercent)?  setVolume,}) {final _that = this;
switch (_that) {
case PlayerCommand_SelectRecording() when selectRecording != null:
return selectRecording(_that.recordingId);case PlayerCommand_Play() when play != null:
return play();case PlayerCommand_Pause() when pause != null:
return pause();case PlayerCommand_Seek() when seek != null:
return seek(_that.positionMs);case PlayerCommand_SetSpeed() when setSpeed != null:
return setSpeed(_that.speed);case PlayerCommand_SetTuning() when setTuning != null:
return setTuning(_that.tuningHz);case PlayerCommand_SetKey() when setKey != null:
return setKey(_that.keyShiftSemitones);case PlayerCommand_SetVolume() when setVolume != null:
return setVolume(_that.volumePercent);case _:
  return null;

}
}

}

/// @nodoc


class PlayerCommand_SelectRecording extends PlayerCommand {
  const PlayerCommand_SelectRecording({required this.recordingId}): super._();
  

 final  BigInt recordingId;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerCommand_SelectRecordingCopyWith<PlayerCommand_SelectRecording> get copyWith => _$PlayerCommand_SelectRecordingCopyWithImpl<PlayerCommand_SelectRecording>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_SelectRecording&&(identical(other.recordingId, recordingId) || other.recordingId == recordingId));
}


@override
int get hashCode {
    return Object.hash(runtimeType,recordingId);
}

@override
String toString() {
    return 'PlayerCommand.selectRecording(recordingId: $recordingId)';
}


}

/// @nodoc
abstract mixin class $PlayerCommand_SelectRecordingCopyWith<$Res> implements $PlayerCommandCopyWith<$Res> {
  factory $PlayerCommand_SelectRecordingCopyWith(PlayerCommand_SelectRecording value, $Res Function(PlayerCommand_SelectRecording) _then) = _$PlayerCommand_SelectRecordingCopyWithImpl;
@useResult
$Res call({
 BigInt recordingId
});




}
/// @nodoc
class _$PlayerCommand_SelectRecordingCopyWithImpl<$Res>
    implements $PlayerCommand_SelectRecordingCopyWith<$Res> {
  _$PlayerCommand_SelectRecordingCopyWithImpl(this._self, this._then);

  final PlayerCommand_SelectRecording _self;
  final $Res Function(PlayerCommand_SelectRecording) _then;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? recordingId = null,}) {
  return _then(PlayerCommand_SelectRecording(
recordingId: null == recordingId ? _self.recordingId : recordingId // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class PlayerCommand_Play extends PlayerCommand {
  const PlayerCommand_Play(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_Play);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayerCommand.play()';
}


}




/// @nodoc


class PlayerCommand_Pause extends PlayerCommand {
  const PlayerCommand_Pause(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_Pause);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayerCommand.pause()';
}


}




/// @nodoc


class PlayerCommand_Seek extends PlayerCommand {
  const PlayerCommand_Seek({required this.positionMs}): super._();
  

 final  BigInt positionMs;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerCommand_SeekCopyWith<PlayerCommand_Seek> get copyWith => _$PlayerCommand_SeekCopyWithImpl<PlayerCommand_Seek>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_Seek&&(identical(other.positionMs, positionMs) || other.positionMs == positionMs));
}


@override
int get hashCode {
    return Object.hash(runtimeType,positionMs);
}

@override
String toString() {
    return 'PlayerCommand.seek(positionMs: $positionMs)';
}


}

/// @nodoc
abstract mixin class $PlayerCommand_SeekCopyWith<$Res> implements $PlayerCommandCopyWith<$Res> {
  factory $PlayerCommand_SeekCopyWith(PlayerCommand_Seek value, $Res Function(PlayerCommand_Seek) _then) = _$PlayerCommand_SeekCopyWithImpl;
@useResult
$Res call({
 BigInt positionMs
});




}
/// @nodoc
class _$PlayerCommand_SeekCopyWithImpl<$Res>
    implements $PlayerCommand_SeekCopyWith<$Res> {
  _$PlayerCommand_SeekCopyWithImpl(this._self, this._then);

  final PlayerCommand_Seek _self;
  final $Res Function(PlayerCommand_Seek) _then;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? positionMs = null,}) {
  return _then(PlayerCommand_Seek(
positionMs: null == positionMs ? _self.positionMs : positionMs // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class PlayerCommand_SetSpeed extends PlayerCommand {
  const PlayerCommand_SetSpeed({required this.speed}): super._();
  

 final  double speed;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerCommand_SetSpeedCopyWith<PlayerCommand_SetSpeed> get copyWith => _$PlayerCommand_SetSpeedCopyWithImpl<PlayerCommand_SetSpeed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_SetSpeed&&(identical(other.speed, speed) || other.speed == speed));
}


@override
int get hashCode {
    return Object.hash(runtimeType,speed);
}

@override
String toString() {
    return 'PlayerCommand.setSpeed(speed: $speed)';
}


}

/// @nodoc
abstract mixin class $PlayerCommand_SetSpeedCopyWith<$Res> implements $PlayerCommandCopyWith<$Res> {
  factory $PlayerCommand_SetSpeedCopyWith(PlayerCommand_SetSpeed value, $Res Function(PlayerCommand_SetSpeed) _then) = _$PlayerCommand_SetSpeedCopyWithImpl;
@useResult
$Res call({
 double speed
});




}
/// @nodoc
class _$PlayerCommand_SetSpeedCopyWithImpl<$Res>
    implements $PlayerCommand_SetSpeedCopyWith<$Res> {
  _$PlayerCommand_SetSpeedCopyWithImpl(this._self, this._then);

  final PlayerCommand_SetSpeed _self;
  final $Res Function(PlayerCommand_SetSpeed) _then;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? speed = null,}) {
  return _then(PlayerCommand_SetSpeed(
speed: null == speed ? _self.speed : speed // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PlayerCommand_SetTuning extends PlayerCommand {
  const PlayerCommand_SetTuning({required this.tuningHz}): super._();
  

 final  double tuningHz;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerCommand_SetTuningCopyWith<PlayerCommand_SetTuning> get copyWith => _$PlayerCommand_SetTuningCopyWithImpl<PlayerCommand_SetTuning>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_SetTuning&&(identical(other.tuningHz, tuningHz) || other.tuningHz == tuningHz));
}


@override
int get hashCode {
    return Object.hash(runtimeType,tuningHz);
}

@override
String toString() {
    return 'PlayerCommand.setTuning(tuningHz: $tuningHz)';
}


}

/// @nodoc
abstract mixin class $PlayerCommand_SetTuningCopyWith<$Res> implements $PlayerCommandCopyWith<$Res> {
  factory $PlayerCommand_SetTuningCopyWith(PlayerCommand_SetTuning value, $Res Function(PlayerCommand_SetTuning) _then) = _$PlayerCommand_SetTuningCopyWithImpl;
@useResult
$Res call({
 double tuningHz
});




}
/// @nodoc
class _$PlayerCommand_SetTuningCopyWithImpl<$Res>
    implements $PlayerCommand_SetTuningCopyWith<$Res> {
  _$PlayerCommand_SetTuningCopyWithImpl(this._self, this._then);

  final PlayerCommand_SetTuning _self;
  final $Res Function(PlayerCommand_SetTuning) _then;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? tuningHz = null,}) {
  return _then(PlayerCommand_SetTuning(
tuningHz: null == tuningHz ? _self.tuningHz : tuningHz // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PlayerCommand_SetKey extends PlayerCommand {
  const PlayerCommand_SetKey({required this.keyShiftSemitones}): super._();
  

 final  int keyShiftSemitones;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerCommand_SetKeyCopyWith<PlayerCommand_SetKey> get copyWith => _$PlayerCommand_SetKeyCopyWithImpl<PlayerCommand_SetKey>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_SetKey&&(identical(other.keyShiftSemitones, keyShiftSemitones) || other.keyShiftSemitones == keyShiftSemitones));
}


@override
int get hashCode {
    return Object.hash(runtimeType,keyShiftSemitones);
}

@override
String toString() {
    return 'PlayerCommand.setKey(keyShiftSemitones: $keyShiftSemitones)';
}


}

/// @nodoc
abstract mixin class $PlayerCommand_SetKeyCopyWith<$Res> implements $PlayerCommandCopyWith<$Res> {
  factory $PlayerCommand_SetKeyCopyWith(PlayerCommand_SetKey value, $Res Function(PlayerCommand_SetKey) _then) = _$PlayerCommand_SetKeyCopyWithImpl;
@useResult
$Res call({
 int keyShiftSemitones
});




}
/// @nodoc
class _$PlayerCommand_SetKeyCopyWithImpl<$Res>
    implements $PlayerCommand_SetKeyCopyWith<$Res> {
  _$PlayerCommand_SetKeyCopyWithImpl(this._self, this._then);

  final PlayerCommand_SetKey _self;
  final $Res Function(PlayerCommand_SetKey) _then;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? keyShiftSemitones = null,}) {
  return _then(PlayerCommand_SetKey(
keyShiftSemitones: null == keyShiftSemitones ? _self.keyShiftSemitones : keyShiftSemitones // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PlayerCommand_SetVolume extends PlayerCommand {
  const PlayerCommand_SetVolume({required this.volumePercent}): super._();
  

 final  int volumePercent;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerCommand_SetVolumeCopyWith<PlayerCommand_SetVolume> get copyWith => _$PlayerCommand_SetVolumeCopyWithImpl<PlayerCommand_SetVolume>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerCommand_SetVolume&&(identical(other.volumePercent, volumePercent) || other.volumePercent == volumePercent));
}


@override
int get hashCode {
    return Object.hash(runtimeType,volumePercent);
}

@override
String toString() {
    return 'PlayerCommand.setVolume(volumePercent: $volumePercent)';
}


}

/// @nodoc
abstract mixin class $PlayerCommand_SetVolumeCopyWith<$Res> implements $PlayerCommandCopyWith<$Res> {
  factory $PlayerCommand_SetVolumeCopyWith(PlayerCommand_SetVolume value, $Res Function(PlayerCommand_SetVolume) _then) = _$PlayerCommand_SetVolumeCopyWithImpl;
@useResult
$Res call({
 int volumePercent
});




}
/// @nodoc
class _$PlayerCommand_SetVolumeCopyWithImpl<$Res>
    implements $PlayerCommand_SetVolumeCopyWith<$Res> {
  _$PlayerCommand_SetVolumeCopyWithImpl(this._self, this._then);

  final PlayerCommand_SetVolume _self;
  final $Res Function(PlayerCommand_SetVolume) _then;

/// Create a copy of PlayerCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? volumePercent = null,}) {
  return _then(PlayerCommand_SetVolume(
volumePercent: null == volumePercent ? _self.volumePercent : volumePercent // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$SessionUpdate {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SessionUpdate);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SessionUpdate()';
}


}

/// @nodoc
class $SessionUpdateCopyWith<$Res>  {
$SessionUpdateCopyWith(SessionUpdate _, $Res Function(SessionUpdate) __);
}


/// Adds pattern-matching-related methods to [SessionUpdate].
extension SessionUpdatePatterns on SessionUpdate {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SessionUpdate_Connection value)?  connection,TResult Function( SessionUpdate_Playback value)?  playback,TResult Function( SessionUpdate_BookmarksChanged value)?  bookmarksChanged,TResult Function( SessionUpdate_AnalysisChanged value)?  analysisChanged,TResult Function( SessionUpdate_Rejected value)?  rejected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SessionUpdate_Connection() when connection != null:
return connection(_that);case SessionUpdate_Playback() when playback != null:
return playback(_that);case SessionUpdate_BookmarksChanged() when bookmarksChanged != null:
return bookmarksChanged(_that);case SessionUpdate_AnalysisChanged() when analysisChanged != null:
return analysisChanged(_that);case SessionUpdate_Rejected() when rejected != null:
return rejected(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SessionUpdate_Connection value)  connection,required TResult Function( SessionUpdate_Playback value)  playback,required TResult Function( SessionUpdate_BookmarksChanged value)  bookmarksChanged,required TResult Function( SessionUpdate_AnalysisChanged value)  analysisChanged,required TResult Function( SessionUpdate_Rejected value)  rejected,}){
final _that = this;
switch (_that) {
case SessionUpdate_Connection():
return connection(_that);case SessionUpdate_Playback():
return playback(_that);case SessionUpdate_BookmarksChanged():
return bookmarksChanged(_that);case SessionUpdate_AnalysisChanged():
return analysisChanged(_that);case SessionUpdate_Rejected():
return rejected(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SessionUpdate_Connection value)?  connection,TResult? Function( SessionUpdate_Playback value)?  playback,TResult? Function( SessionUpdate_BookmarksChanged value)?  bookmarksChanged,TResult? Function( SessionUpdate_AnalysisChanged value)?  analysisChanged,TResult? Function( SessionUpdate_Rejected value)?  rejected,}){
final _that = this;
switch (_that) {
case SessionUpdate_Connection() when connection != null:
return connection(_that);case SessionUpdate_Playback() when playback != null:
return playback(_that);case SessionUpdate_BookmarksChanged() when bookmarksChanged != null:
return bookmarksChanged(_that);case SessionUpdate_AnalysisChanged() when analysisChanged != null:
return analysisChanged(_that);case SessionUpdate_Rejected() when rejected != null:
return rejected(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ConnectionState field0)?  connection,TResult Function( PlaybackView field0)?  playback,TResult Function( BigInt recordingId)?  bookmarksChanged,TResult Function( BigInt recordingId)?  analysisChanged,TResult Function( String reason)?  rejected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SessionUpdate_Connection() when connection != null:
return connection(_that.field0);case SessionUpdate_Playback() when playback != null:
return playback(_that.field0);case SessionUpdate_BookmarksChanged() when bookmarksChanged != null:
return bookmarksChanged(_that.recordingId);case SessionUpdate_AnalysisChanged() when analysisChanged != null:
return analysisChanged(_that.recordingId);case SessionUpdate_Rejected() when rejected != null:
return rejected(_that.reason);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ConnectionState field0)  connection,required TResult Function( PlaybackView field0)  playback,required TResult Function( BigInt recordingId)  bookmarksChanged,required TResult Function( BigInt recordingId)  analysisChanged,required TResult Function( String reason)  rejected,}) {final _that = this;
switch (_that) {
case SessionUpdate_Connection():
return connection(_that.field0);case SessionUpdate_Playback():
return playback(_that.field0);case SessionUpdate_BookmarksChanged():
return bookmarksChanged(_that.recordingId);case SessionUpdate_AnalysisChanged():
return analysisChanged(_that.recordingId);case SessionUpdate_Rejected():
return rejected(_that.reason);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ConnectionState field0)?  connection,TResult? Function( PlaybackView field0)?  playback,TResult? Function( BigInt recordingId)?  bookmarksChanged,TResult? Function( BigInt recordingId)?  analysisChanged,TResult? Function( String reason)?  rejected,}) {final _that = this;
switch (_that) {
case SessionUpdate_Connection() when connection != null:
return connection(_that.field0);case SessionUpdate_Playback() when playback != null:
return playback(_that.field0);case SessionUpdate_BookmarksChanged() when bookmarksChanged != null:
return bookmarksChanged(_that.recordingId);case SessionUpdate_AnalysisChanged() when analysisChanged != null:
return analysisChanged(_that.recordingId);case SessionUpdate_Rejected() when rejected != null:
return rejected(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class SessionUpdate_Connection extends SessionUpdate {
  const SessionUpdate_Connection(this.field0): super._();
  

 final  ConnectionState field0;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SessionUpdate_ConnectionCopyWith<SessionUpdate_Connection> get copyWith => _$SessionUpdate_ConnectionCopyWithImpl<SessionUpdate_Connection>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SessionUpdate_Connection&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'SessionUpdate.connection(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $SessionUpdate_ConnectionCopyWith<$Res> implements $SessionUpdateCopyWith<$Res> {
  factory $SessionUpdate_ConnectionCopyWith(SessionUpdate_Connection value, $Res Function(SessionUpdate_Connection) _then) = _$SessionUpdate_ConnectionCopyWithImpl;
@useResult
$Res call({
 ConnectionState field0
});




}
/// @nodoc
class _$SessionUpdate_ConnectionCopyWithImpl<$Res>
    implements $SessionUpdate_ConnectionCopyWith<$Res> {
  _$SessionUpdate_ConnectionCopyWithImpl(this._self, this._then);

  final SessionUpdate_Connection _self;
  final $Res Function(SessionUpdate_Connection) _then;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(SessionUpdate_Connection(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as ConnectionState,
  ));
}


}

/// @nodoc


class SessionUpdate_Playback extends SessionUpdate {
  const SessionUpdate_Playback(this.field0): super._();
  

 final  PlaybackView field0;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SessionUpdate_PlaybackCopyWith<SessionUpdate_Playback> get copyWith => _$SessionUpdate_PlaybackCopyWithImpl<SessionUpdate_Playback>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SessionUpdate_Playback&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'SessionUpdate.playback(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $SessionUpdate_PlaybackCopyWith<$Res> implements $SessionUpdateCopyWith<$Res> {
  factory $SessionUpdate_PlaybackCopyWith(SessionUpdate_Playback value, $Res Function(SessionUpdate_Playback) _then) = _$SessionUpdate_PlaybackCopyWithImpl;
@useResult
$Res call({
 PlaybackView field0
});




}
/// @nodoc
class _$SessionUpdate_PlaybackCopyWithImpl<$Res>
    implements $SessionUpdate_PlaybackCopyWith<$Res> {
  _$SessionUpdate_PlaybackCopyWithImpl(this._self, this._then);

  final SessionUpdate_Playback _self;
  final $Res Function(SessionUpdate_Playback) _then;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(SessionUpdate_Playback(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as PlaybackView,
  ));
}


}

/// @nodoc


class SessionUpdate_BookmarksChanged extends SessionUpdate {
  const SessionUpdate_BookmarksChanged({required this.recordingId}): super._();
  

 final  BigInt recordingId;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SessionUpdate_BookmarksChangedCopyWith<SessionUpdate_BookmarksChanged> get copyWith => _$SessionUpdate_BookmarksChangedCopyWithImpl<SessionUpdate_BookmarksChanged>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SessionUpdate_BookmarksChanged&&(identical(other.recordingId, recordingId) || other.recordingId == recordingId));
}


@override
int get hashCode {
    return Object.hash(runtimeType,recordingId);
}

@override
String toString() {
    return 'SessionUpdate.bookmarksChanged(recordingId: $recordingId)';
}


}

/// @nodoc
abstract mixin class $SessionUpdate_BookmarksChangedCopyWith<$Res> implements $SessionUpdateCopyWith<$Res> {
  factory $SessionUpdate_BookmarksChangedCopyWith(SessionUpdate_BookmarksChanged value, $Res Function(SessionUpdate_BookmarksChanged) _then) = _$SessionUpdate_BookmarksChangedCopyWithImpl;
@useResult
$Res call({
 BigInt recordingId
});




}
/// @nodoc
class _$SessionUpdate_BookmarksChangedCopyWithImpl<$Res>
    implements $SessionUpdate_BookmarksChangedCopyWith<$Res> {
  _$SessionUpdate_BookmarksChangedCopyWithImpl(this._self, this._then);

  final SessionUpdate_BookmarksChanged _self;
  final $Res Function(SessionUpdate_BookmarksChanged) _then;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? recordingId = null,}) {
  return _then(SessionUpdate_BookmarksChanged(
recordingId: null == recordingId ? _self.recordingId : recordingId // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class SessionUpdate_AnalysisChanged extends SessionUpdate {
  const SessionUpdate_AnalysisChanged({required this.recordingId}): super._();
  

 final  BigInt recordingId;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SessionUpdate_AnalysisChangedCopyWith<SessionUpdate_AnalysisChanged> get copyWith => _$SessionUpdate_AnalysisChangedCopyWithImpl<SessionUpdate_AnalysisChanged>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SessionUpdate_AnalysisChanged&&(identical(other.recordingId, recordingId) || other.recordingId == recordingId));
}


@override
int get hashCode {
    return Object.hash(runtimeType,recordingId);
}

@override
String toString() {
    return 'SessionUpdate.analysisChanged(recordingId: $recordingId)';
}


}

/// @nodoc
abstract mixin class $SessionUpdate_AnalysisChangedCopyWith<$Res> implements $SessionUpdateCopyWith<$Res> {
  factory $SessionUpdate_AnalysisChangedCopyWith(SessionUpdate_AnalysisChanged value, $Res Function(SessionUpdate_AnalysisChanged) _then) = _$SessionUpdate_AnalysisChangedCopyWithImpl;
@useResult
$Res call({
 BigInt recordingId
});




}
/// @nodoc
class _$SessionUpdate_AnalysisChangedCopyWithImpl<$Res>
    implements $SessionUpdate_AnalysisChangedCopyWith<$Res> {
  _$SessionUpdate_AnalysisChangedCopyWithImpl(this._self, this._then);

  final SessionUpdate_AnalysisChanged _self;
  final $Res Function(SessionUpdate_AnalysisChanged) _then;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? recordingId = null,}) {
  return _then(SessionUpdate_AnalysisChanged(
recordingId: null == recordingId ? _self.recordingId : recordingId // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class SessionUpdate_Rejected extends SessionUpdate {
  const SessionUpdate_Rejected({required this.reason}): super._();
  

 final  String reason;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SessionUpdate_RejectedCopyWith<SessionUpdate_Rejected> get copyWith => _$SessionUpdate_RejectedCopyWithImpl<SessionUpdate_Rejected>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SessionUpdate_Rejected&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode {
    return Object.hash(runtimeType,reason);
}

@override
String toString() {
    return 'SessionUpdate.rejected(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $SessionUpdate_RejectedCopyWith<$Res> implements $SessionUpdateCopyWith<$Res> {
  factory $SessionUpdate_RejectedCopyWith(SessionUpdate_Rejected value, $Res Function(SessionUpdate_Rejected) _then) = _$SessionUpdate_RejectedCopyWithImpl;
@useResult
$Res call({
 String reason
});




}
/// @nodoc
class _$SessionUpdate_RejectedCopyWithImpl<$Res>
    implements $SessionUpdate_RejectedCopyWith<$Res> {
  _$SessionUpdate_RejectedCopyWithImpl(this._self, this._then);

  final SessionUpdate_Rejected _self;
  final $Res Function(SessionUpdate_Rejected) _then;

/// Create a copy of SessionUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(SessionUpdate_Rejected(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
