use bspffi::types::XCSlice;

mod geometry;
mod parsing;

pub use geometry::{DPlane3, DVec2, DVec3, DVec4};
pub use parsing::{LineCounter, ParseError, ParseResult};

pub type XCBytes<'l> = XCSlice<'l, u8>;
