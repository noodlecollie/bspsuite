mod geometry;
mod parsing;
mod portable_option;
mod slice_ref;
mod string_ref;

pub use geometry::{DPlane, DVec2, DVec3, DVec4};
pub use parsing::{LineCounter, ParseError, ParseResult};
pub use portable_option::PortableOption;
pub use slice_ref::SliceRef;
pub use string_ref::StringRef;

pub type BytesRef<'l> = SliceRef<'l, u8>;
