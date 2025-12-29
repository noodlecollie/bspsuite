/// Trait implemented by objects which marshal a reference to a native value, so
/// that it can be communicated across an FFI boundary.
///
/// The type implementing `RefMarshaller` is expected to be FFI-safe and `extern
/// "C"`-safe. It wraps a reference to a native value using `marshal_ref()`, and
/// after the marshaller has crossed a library boundary, a reference to its
/// marshalled value can be obtained using `unmarshal_ref()`.
///
/// This trait does not encode any requirements about how the marshalling takes
/// place. Its main use is to indicate that an implementer type is a portable
/// alias for some native type. This information can be used to easily extract a
/// reference to a value of this native type. `XCOption` uses this to, for
/// example, easily obtain a `&str` from an `XCStr`.
pub trait RefMarshaller<'l, NativeType>
where
	NativeType: ?Sized,
{
	/// Given a reference to a `value` of `NativeType`, wraps the reference and
	/// returns an instance of the marshaller. This marshaller can be used to
	/// unmarshal the reference later with [unmarshal_ref].
	fn marshal_ref(value: &'l NativeType) -> Self;

	/// Returns a reference to a value previously marshalled with [marshal_ref].
	fn unmarshal_ref(&'l self) -> &'l NativeType;
}
