mod geometry;
mod parsing;
mod slice_ref;

pub use geometry::{DPlane, DVec2, DVec3, DVec4};
pub use parsing::{LineCounter, ParseError, ParseResult};
pub use slice_ref::SliceRef;

pub type BytesRef<'l> = SliceRef<'l, u8>;
