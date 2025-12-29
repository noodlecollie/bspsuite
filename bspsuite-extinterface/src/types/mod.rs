use bspsuite_ffim::types::XCSlice;

mod geometry;
mod parsing;

pub use geometry::{DPlane, DVec2, DVec3, DVec4};
pub use parsing::{LineCounter, ParseError, ParseResult};

pub type XCBytes<'l> = XCSlice<'l, u8>;
