use crate::math::CompileTuningParameters;
use crate::model::{MapCsgBrush, MapCsgBrushFace, MapSourceBrush, MapSourceBrushFace};
use anyhow::Result;

// In combination with the Stefan Hajnoczi paper (see the notes directory in
// this repo), and with
// https://math.stackexchange.com/questions/5120422/how-to-verify-whether-a-collection-of-3d-planes-forms-a-valid-convex-hull/5120722,
// the approach we follow here is:
// For each plane (face):
//   Check intersections with all other planes. For each intersection (edge):
//     Check intersections with all remaining planes to create vertex list.
//     Remove extraneous vertices (in front of any planes).
//     If edge vertices != 2, solid is not valid.
//     Add vertices and edges to respective faces.
//   If faace vertices < 3, solid is not valid.
//   Order face's vertices clockwise, using edges to link vertices.
//   Compute texture co-ordinates for each vertex.
// Normalise texture co-ordinates for all vertices in brush.
pub fn construct_brush(
	source: MapSourceBrush,
	params: &CompileTuningParameters,
) -> Result<MapCsgBrush>
{
	let faces: Vec<MapCsgBrushFace> = create_faces_from_planes(&source.faces);
	todo!();
}

fn create_faces_from_planes(brush_faces: &Vec<MapSourceBrushFace>) -> Vec<MapCsgBrushFace>
{
	todo!();
}
