use crate::map_formats::map_valve220::parse_map;
use bspextifc::containers::map_blueprint_builder::{
	Brush, BrushFace, BuilderError, Entity, MapBlueprintBuilder,
};
use bspextifc::types::{DPlane, DVec2, DVec3};

const BOX_MAP_SOURCE: &str = r#"
// entity 0
{
	"classname" "worldspawn"
	"mapversion" "220"
	// brush 0
	{
	( -256 -256 0 ) ( -256 -248 0 ) ( -256 -248 200 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 256 -256 0 ) ( -256 -256 0 ) ( -256 -256 200 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -256 -256 0 ) ( 256 -256 0 ) ( 256 -248 0 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -256 -256 200 ) ( -256 -248 200 ) ( 256 -248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -256 -248 0 ) ( 256 -248 0 ) ( 256 -248 200 ) knox/ctf_bconcrete_1 [ 1 0 0 0 ] [ 0 0 -1 256 ] 0 1.25 1.25
	( 256 -256 0 ) ( 256 -256 200 ) ( 256 -248 200 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	}
	// brush 1
	{
	( -256 256 0 ) ( -256 256 200 ) ( -256 248 200 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 256 248 0 ) ( -256 248 0 ) ( -256 248 200 ) knox/ctf_bconcrete_1 [ 1 0 0 0 ] [ 0 0 -1 256 ] 0 1.25 1.25
	( -256 256 0 ) ( -256 248 0 ) ( 256 248 0 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -256 256 200 ) ( 256 256 200 ) ( 256 248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -256 256 0 ) ( 256 256 0 ) ( 256 256 200 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 256 256 0 ) ( 256 248 0 ) ( 256 248 200 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	}
	// brush 2
	{
	( -256 248 0 ) ( -256 248 200 ) ( -256 -248 200 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -256 -248 0 ) ( -256 -248 200 ) ( -248 -248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -256 -248 0 ) ( -248 -248 0 ) ( -248 248 0 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -256 248 200 ) ( -248 248 200 ) ( -248 -248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -256 248 0 ) ( -248 248 0 ) ( -248 248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -248 -248 0 ) ( -248 -248 200 ) ( -248 248 200 ) knox/ctf_bconcrete_1 [ 0 1 0 0 ] [ 0 0 -1 256 ] 0 1.25 1.25
	}
	// brush 3
	{
	( 248 248 0 ) ( 248 248 200 ) ( 248 -248 200 ) knox/ctf_bconcrete_1 [ 0 1 0 0 ] [ 0 0 -1 256 ] 0 1.25 1.25
	( 256 -248 0 ) ( 248 -248 0 ) ( 248 -248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 256 -248 0 ) ( 256 248 0 ) ( 248 248 0 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( 256 248 200 ) ( 256 -248 200 ) ( 248 -248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( 256 248 0 ) ( 256 248 200 ) ( 248 248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 256 -248 0 ) ( 256 -248 200 ) ( 256 248 200 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	}
	// brush 4
	{
	( -248 248 200 ) ( -248 -248 200 ) ( -248 -248 192 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -248 -248 200 ) ( 248 -248 200 ) ( 248 -248 192 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -248 248 192 ) ( -248 -248 192 ) ( 248 -248 192 ) island/isle_concrete [ 1 0 0 0 ] [ 0 -1 0 256 ] 0 1.25 1.25
	( -248 -248 200 ) ( -248 248 200 ) ( 248 248 200 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( 248 248 200 ) ( -248 248 200 ) ( -248 248 192 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 248 -248 200 ) ( 248 248 200 ) ( 248 248 192 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	}
	// brush 5
	{
	( -248 -248 0 ) ( -248 248 0 ) ( -248 248 8 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 248 -248 0 ) ( -248 -248 0 ) ( -248 -248 8 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( -248 248 0 ) ( -248 -248 0 ) ( 248 -248 0 ) special/nodraw [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -248 -248 8 ) ( -248 248 8 ) ( 248 248 8 ) power/concfloor01 [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
	( -248 248 0 ) ( 248 248 0 ) ( 248 248 8 ) special/nodraw [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
	( 248 248 0 ) ( 248 -248 0 ) ( 248 -248 8 ) special/nodraw [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
	}
}
// entity 1
{
	"classname" "info_player_start"
	"origin" "126 85 45"
}
"#;

#[test]
fn parse_box_map()
{
	let mut builder: MapBlueprintBuilder = MapBlueprintBuilder::new();
	let result = parse_map(BOX_MAP_SOURCE, &mut builder);
	let build_result: Result<Vec<Entity>, BuilderError> = builder.collect();

	assert!(result.is_ok());
	assert!(build_result.is_ok());

	let build_result = build_result.unwrap();

	let world: &Entity = &build_result[0];
	assert_eq!(world.keyvalues.len(), 2);
	assert_eq!(
		world.keyvalues.get("classname"),
		Some("worldspawn".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("mapversion"),
		Some("220".to_owned()).as_ref()
	);

	let player_start: &Entity = &build_result[1];
	assert_eq!(player_start.keyvalues.len(), 2);
	assert_eq!(
		player_start.keyvalues.get("classname"),
		Some("info_player_start".to_owned()).as_ref()
	);
	assert_eq!(
		player_start.keyvalues.get("origin"),
		Some("126 85 45".to_owned()).as_ref()
	);

	let brushes: &Vec<Brush> = &world.brushes;
	assert_eq!(brushes.len(), 6);

	// TODO: Check each brush
	check_brush_0(&brushes[0]);
	check_brush_1(&brushes[1]);
	check_brush_2(&brushes[2]);
}

fn check_brush_0(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 200.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, -248.0),
			material_name: "knox/ctf_bconcrete_1".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 256.0),
			material_scale: DVec2::new(1.25, 1.25),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);
}

fn check_brush_1(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, -248.0),
			material_name: "knox/ctf_bconcrete_1".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 256.0),
			material_scale: DVec2::new(1.25, 1.25),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 200.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);
}

fn check_brush_2(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, 256.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 248.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 200.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 248.0),
			material_name: "special/nodraw".to_owned(),
			material_axes: (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, -248.0),
			material_name: "knox/ctf_bconcrete_1".to_owned(),
			material_axes: (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			material_offset: DVec2::new(0.0, 256.0),
			material_scale: DVec2::new(1.25, 1.25),
		},
	);
}

fn check_brush_face(actual: &BrushFace, expected: &BrushFace)
{
	assert_eq!(expected.plane, actual.plane);
	assert_eq!(expected.material_name, actual.material_name);
	assert_eq!(expected.material_axes.0, actual.material_axes.0);
	assert_eq!(expected.material_axes.1, actual.material_axes.1);
	assert_eq!(expected.material_offset, actual.material_offset);
	assert_eq!(expected.material_scale, actual.material_scale);
}

enum PlaneDir
{
	Pos,
	Neg,
}

impl PlaneDir
{
	pub fn as_number(&self) -> f64
	{
		return match self
		{
			PlaneDir::Pos => 1.0,
			PlaneDir::Neg => -1.0,
		};
	}
}

fn x_plane(dir: PlaneDir, dist: f64) -> DPlane
{
	return DPlane::new(DVec3::new(dir.as_number(), 0.0, 0.0), dist);
}

fn y_plane(dir: PlaneDir, dist: f64) -> DPlane
{
	return DPlane::new(DVec3::new(0.0, dir.as_number(), 0.0), dist);
}

fn z_plane(dir: PlaneDir, dist: f64) -> DPlane
{
	return DPlane::new(DVec3::new(0.0, 0.0, dir.as_number()), dist);
}
