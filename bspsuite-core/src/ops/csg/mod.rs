mod csg_brush_builder;
mod edge_collection;
mod point_collection;

use crate::math::CompileTuningParameters;
use crate::model::{MapGeomBrush, MapSourceBrush};
use anyhow::Result;
use csg_brush_builder::CsgBrushBuilder;

pub fn construct_brush(
	source: MapSourceBrush,
	params: &CompileTuningParameters,
) -> Result<MapGeomBrush>
{
	return CsgBrushBuilder::build(&source, params);
}
