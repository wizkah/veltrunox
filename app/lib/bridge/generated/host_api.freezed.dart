// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'host_api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SharingProblem {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SharingProblem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SharingProblem()';
}


}

/// @nodoc
class $SharingProblemCopyWith<$Res>  {
$SharingProblemCopyWith(SharingProblem _, $Res Function(SharingProblem) __);
}


/// Adds pattern-matching-related methods to [SharingProblem].
extension SharingProblemPatterns on SharingProblem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SharingProblem_NoAccess value)?  noAccess,TResult Function( SharingProblem_PortInUse value)?  portInUse,TResult Function( SharingProblem_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SharingProblem_NoAccess() when noAccess != null:
return noAccess(_that);case SharingProblem_PortInUse() when portInUse != null:
return portInUse(_that);case SharingProblem_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SharingProblem_NoAccess value)  noAccess,required TResult Function( SharingProblem_PortInUse value)  portInUse,required TResult Function( SharingProblem_Failed value)  failed,}){
final _that = this;
switch (_that) {
case SharingProblem_NoAccess():
return noAccess(_that);case SharingProblem_PortInUse():
return portInUse(_that);case SharingProblem_Failed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SharingProblem_NoAccess value)?  noAccess,TResult? Function( SharingProblem_PortInUse value)?  portInUse,TResult? Function( SharingProblem_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case SharingProblem_NoAccess() when noAccess != null:
return noAccess(_that);case SharingProblem_PortInUse() when portInUse != null:
return portInUse(_that);case SharingProblem_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  noAccess,TResult Function()?  portInUse,TResult Function( String detail)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SharingProblem_NoAccess() when noAccess != null:
return noAccess();case SharingProblem_PortInUse() when portInUse != null:
return portInUse();case SharingProblem_Failed() when failed != null:
return failed(_that.detail);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  noAccess,required TResult Function()  portInUse,required TResult Function( String detail)  failed,}) {final _that = this;
switch (_that) {
case SharingProblem_NoAccess():
return noAccess();case SharingProblem_PortInUse():
return portInUse();case SharingProblem_Failed():
return failed(_that.detail);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  noAccess,TResult? Function()?  portInUse,TResult? Function( String detail)?  failed,}) {final _that = this;
switch (_that) {
case SharingProblem_NoAccess() when noAccess != null:
return noAccess();case SharingProblem_PortInUse() when portInUse != null:
return portInUse();case SharingProblem_Failed() when failed != null:
return failed(_that.detail);case _:
  return null;

}
}

}

/// @nodoc


class SharingProblem_NoAccess extends SharingProblem {
  const SharingProblem_NoAccess(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SharingProblem_NoAccess);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SharingProblem.noAccess()';
}


}




/// @nodoc


class SharingProblem_PortInUse extends SharingProblem {
  const SharingProblem_PortInUse(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SharingProblem_PortInUse);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SharingProblem.portInUse()';
}


}




/// @nodoc


class SharingProblem_Failed extends SharingProblem {
  const SharingProblem_Failed({required this.detail}): super._();
  

 final  String detail;

/// Create a copy of SharingProblem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharingProblem_FailedCopyWith<SharingProblem_Failed> get copyWith => _$SharingProblem_FailedCopyWithImpl<SharingProblem_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SharingProblem_Failed&&(identical(other.detail, detail) || other.detail == detail));
}


@override
int get hashCode {
    return Object.hash(runtimeType,detail);
}

@override
String toString() {
    return 'SharingProblem.failed(detail: $detail)';
}


}

/// @nodoc
abstract mixin class $SharingProblem_FailedCopyWith<$Res> implements $SharingProblemCopyWith<$Res> {
  factory $SharingProblem_FailedCopyWith(SharingProblem_Failed value, $Res Function(SharingProblem_Failed) _then) = _$SharingProblem_FailedCopyWithImpl;
@useResult
$Res call({
 String detail
});




}
/// @nodoc
class _$SharingProblem_FailedCopyWithImpl<$Res>
    implements $SharingProblem_FailedCopyWith<$Res> {
  _$SharingProblem_FailedCopyWithImpl(this._self, this._then);

  final SharingProblem_Failed _self;
  final $Res Function(SharingProblem_Failed) _then;

/// Create a copy of SharingProblem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? detail = null,}) {
  return _then(SharingProblem_Failed(
detail: null == detail ? _self.detail : detail // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
