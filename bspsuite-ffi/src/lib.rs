//! Crate containing helper structs and routines for FFI inter-operation.
//!
//! The BSPSuite compiler is built to be modular, and as such makes heavy use of
//! dynamic libraries and `extern "C"` functions. There are a couple of
//! fundamental requirements that must be fulfilled in order to make this work:
//!
//! 1. Exchange of significant amounts of data across library boundaries - for
//!    example, when parsing a map source file in an extension library, and
//!    passing the geometry data back to the main compiler.
//! 2. Ergonomic function call interaction between the compiler and its
//!    extensions - for example, an extension should be able to call functions
//!    on a struct reference it has been passed, rather than needing to know
//!    about any underlying C-style or unsafe functions.
//!
//! This crate provides helpers in order to ease the inter-operation between the
//! main compiler and its extension libraries.
//!
//! Helper types are found under the `types` module. These types are designed to
//! be used as `extern "C"` function arguments, and wrap unsafe behaviour so
//! that the compiler and its extensions don't need to worry about it. These
//! types are prefixed `XC` to indicate `extern "C"` compatibility.
//!
//! Note that `XC` types are not designed to be used as general-purpose data
//! types. They are designed to serve as portable references to existing data,
//! and should be converted back to proper data values or references before the
//! data is used. For this reason, types like `XCOption` that mimic existing
//! Rust types do not implement the suite of functions that the native Rust type
//! does. It is expected that, for example, an `XCOption` would be converted to
//! an `&Option` in order to use functions like `map()`.

pub mod traits;
pub mod types;
