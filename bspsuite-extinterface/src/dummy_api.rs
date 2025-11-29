// Example of the conventions used to create an extension API.

use super::api_info::ApiInfo;
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

// Each API has a name and a version.
pub const API_INFO: ApiInfo = ApiInfo::new("DummyApi", 1);

// Alias for the entry point function signature.
// The core API calls a function like this on the
// extension in order to run extension code for
// this API.
pub type EntryPointFn = extern "C" fn(&mut DummyApi);

// The functions an extension can call to interact with the
// core library are on a struct named "<api name>Api".
#[repr(C)]
pub struct DummyApi<'l>
{
	// This struct owns the internal, unsafe implementation of the
	// functions defined in the core API, and it wraps them for
	// the extension to call them.
	fns: internal::DummyApiCoreFns<'l>,
}

// The functions the core library can call to interact with
// the extension are on a struct named "<api name>Callbacks".
#[repr(C)]
#[derive(Clone)]
pub struct DummyCallbacks
{
	// This function is called by the core library.
	// It executes code in the extension library.
	// A mutable reference to the API functions is provided.
	pub entry_point: EntryPointFn,
}

// Shim wrapper functions for calling into the core library from
// the extension.
impl<'l> DummyApi<'l>
{
	// Store a number in the core library.
	pub fn store_number(&mut self, value: i32)
	{
		// SAFETY: Core library responsible for ensuring that self.fns.context
		// is valid for this struct's lifetime, and that the function being
		// called knows what type to convert the context into.
		unsafe { (self.fns.store_number_fn)(*self.fns.context, value) };
	}

	// Get a number from the core library.
	pub fn get_magic_number(&self) -> i32
	{
		// SAFETY: Core library responsible for ensuring that self.fns.context
		// is valid for this struct's lifetime, and that the function being
		// called knows what type to convert the context into.
		return unsafe { (self.fns.get_magic_number_fn)(*self.fns.context) };
	}
}

pub mod internal
{
	use super::*;

	// This struct is filled out by the core library.
	// SAFETY:
	// The creator of this struct must guarantee:
	// - The data pointed to by the context pointer lives for at least as long as
	//   this struct lives. This is implied by the reference lifetime.
	// - Functions stored in the struct convert the context pointer to the correct
	//   type before they use it.
	#[repr(C)]
	pub struct DummyApiCoreFns<'l>
	{
		// Arbitrary context pointer for the core library functions.
		// The lifetime indicates that the context must live at least
		// as long as this struct does.
		pub context: &'l *mut c_void,

		// Functions implemented by the core library.
		pub store_number_fn: unsafe extern "C" fn(*mut c_void, i32),
		pub get_magic_number_fn: unsafe extern "C" fn(*const c_void) -> i32,
	}

	// Called by the core library in order to create the dummy API struct.
	pub fn create_dummy_api<'l>(fns: internal::DummyApiCoreFns<'l>) -> DummyApi<'l>
	{
		return DummyApi { fns: fns };
	}
}
