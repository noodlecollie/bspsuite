mod csg_brush_builder;
mod edge_collection;
mod point_collection;

use crate::math::CompileTuningParameters;
use crate::model::{MapCsgBrush, MapSourceBrush};
use anyhow::Result;
use csg_brush_builder::CsgBrushBuilder;

pub fn construct_brush(
	source: MapSourceBrush,
	params: &CompileTuningParameters,
) -> Result<MapCsgBrush>
{
	return CsgBrushBuilder::build(&source, params);
}
