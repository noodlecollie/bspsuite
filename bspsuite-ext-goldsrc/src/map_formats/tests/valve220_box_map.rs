use crate::map_formats::map_valve220::parse_map;
use bspextifc::builders::map_blueprint_builder::{
	Brush, BrushFace, BuilderError, Entity, MapBlueprintBuilder,
};
use bspextifc::types::{DPlane, DVec2, DVec3};

const BOX_MAP_SOURCE: &str = r#"
// entity 0
{
"mapversion" "220"
"wad" "halflife.wad"
"classname" "worldspawn"
"sounds" "1"
"MaxRange" "4096"
"startdark" "0"
"gametitle" "0"
"newunit" "0"
"defaultteam" "0"
// brush 0
{
// Manually throw some curveball textures in here, to make sure we support their names.
( -64 -64 -32 ) ( -64 -63 -32 ) ( -64 -64 -31 ) {MASKED [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -64 -32 ) ( -64 -64 -31 ) ( -63 -64 -32 ) !LIQUID [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -64 -16 ) ( -63 -64 -16 ) ( -64 -63 -16 ) +ANIMATED [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 64 0 ) ( 64 65 0 ) ( 65 64 0 ) C1A0_LABFLRE [ 1 0 0 16 ] [ 0 -1 0 -32 ] 0 0.5 0.5
( 64 64 0 ) ( 65 64 0 ) ( 64 64 1 ) -TILING [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 64 64 0 ) ( 64 64 1 ) ( 64 65 0 ) C1A0_LABGLU [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
// brush 1
{
( -64 64 0 ) ( -64 65 0 ) ( -64 64 1 ) C1A0_LABGLU [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 64 0 ) ( -64 64 1 ) ( -63 64 0 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 64 0 ) ( -63 64 0 ) ( -64 65 0 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 192 128 ) ( 64 193 128 ) ( 65 192 128 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 80 32 ) ( 65 80 32 ) ( 64 80 33 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 64 192 32 ) ( 64 192 33 ) ( 64 193 32 ) C1A0_LABGLU [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
// brush 2
{
( -64 -80 0 ) ( -64 -79 0 ) ( -64 -80 1 ) C1A0_LABGLU [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -80 0 ) ( -64 -80 1 ) ( -63 -80 0 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -80 0 ) ( -63 -80 0 ) ( -64 -79 0 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 48 128 ) ( 64 49 128 ) ( 65 48 128 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 -64 32 ) ( 65 -64 32 ) ( 64 -64 33 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 64 48 32 ) ( 64 48 33 ) ( 64 49 32 ) C1A0_LABGLU [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
// brush 3
{
( -80 -64 0 ) ( -80 -63 0 ) ( -80 -64 1 ) C1A0_LABGLU [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -80 -64 0 ) ( -80 -64 1 ) ( -79 -64 0 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -80 -64 0 ) ( -79 -64 0 ) ( -80 -63 0 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 48 64 128 ) ( 48 65 128 ) ( 49 64 128 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 48 64 32 ) ( 49 64 32 ) ( 48 64 33 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 64 32 ) ( -64 64 33 ) ( -64 65 32 ) C1A0_LABGLU [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
// brush 4
{
( 64 -64 0 ) ( 64 -63 0 ) ( 64 -64 1 ) C1A0_LABGLU [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 64 -64 0 ) ( 64 -64 1 ) ( 65 -64 0 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 64 -64 0 ) ( 65 -64 0 ) ( 64 -63 0 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 192 64 128 ) ( 192 65 128 ) ( 193 64 128 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 192 64 32 ) ( 193 64 32 ) ( 192 64 33 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 80 64 32 ) ( 80 64 33 ) ( 80 65 32 ) C1A0_LABGLU [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
// brush 5
{
( -64 -64 112 ) ( -64 -63 112 ) ( -64 -64 113 ) C1A0_LABGLU [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -64 112 ) ( -64 -64 113 ) ( -63 -64 112 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -64 128 ) ( -63 -64 128 ) ( -64 -63 128 ) C1A0_W2 [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 64 144 ) ( 64 65 144 ) ( 65 64 144 ) C1A0_LABGLU [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 64 144 ) ( 65 64 144 ) ( 64 64 145 ) C1A0_LABGLU [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 64 64 144 ) ( 64 64 145 ) ( 64 65 144 ) C1A0_LABGLU [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
}
// entity 1
{
"classname" "info_player_start"
"spawnflags" "0"
"angles" "0 0 0"
"origin" "-16 0 36"
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
	assert_eq!(world.keyvalues.len(), 9);
	assert_eq!(
		world.keyvalues.get("classname"),
		Some("worldspawn".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("mapversion"),
		Some("220".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("wad"),
		Some("halflife.wad".to_owned()).as_ref()
	);
	assert_eq!(world.keyvalues.get("sounds"), Some("1".to_owned()).as_ref());
	assert_eq!(
		world.keyvalues.get("MaxRange"),
		Some("4096".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("startdark"),
		Some("0".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("gametitle"),
		Some("0".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("newunit"),
		Some("0".to_owned()).as_ref()
	);
	assert_eq!(
		world.keyvalues.get("defaultteam"),
		Some("0".to_owned()).as_ref()
	);

	let player_start: &Entity = &build_result[1];
	assert_eq!(player_start.keyvalues.len(), 4);
	assert_eq!(
		player_start.keyvalues.get("classname"),
		Some("info_player_start".to_owned()).as_ref()
	);
	assert_eq!(
		player_start.keyvalues.get("origin"),
		Some("-16 0 36".to_owned()).as_ref()
	);
	assert_eq!(
		player_start.keyvalues.get("angles"),
		Some("0 0 0".to_owned()).as_ref()
	);
	assert_eq!(
		player_start.keyvalues.get("spawnflags"),
		Some("0".to_owned()).as_ref()
	);

	let brushes: &Vec<Brush> = &world.brushes;
	assert_eq!(brushes.len(), 6);

	check_brush_0(&brushes[0]);
	check_brush_1(&brushes[1]);
	check_brush_2(&brushes[2]);
	check_brush_3(&brushes[3]);
	check_brush_4(&brushes[4]);
	check_brush_5(&brushes[5]);
}

fn check_brush_0(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, 64.0),
			material_name: "{MASKED".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 64.0),
			material_name: "!LIQUID".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 16.0),
			material_name: "+ANIMATED".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 0.0),
			material_name: "C1A0_LABFLRE".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_z(),
			material_offset: DVec2::new(16.0, -32.0),
			material_scale: DVec2::new(0.5, 0.5),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 64.0),
			material_name: "-TILING".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_x(),
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
			plane: x_plane(PlaneDir::Neg, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, -64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 128.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 80.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_x(),
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
			plane: x_plane(PlaneDir::Neg, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 80.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 128.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, -64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);
}

fn check_brush_3(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, 80.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 128.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, -64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);
}

fn check_brush_4(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, -64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, 0.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 128.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 80.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);
}

fn check_brush_5(actual: &Brush)
{
	assert_eq!(actual.faces.len(), 6);

	check_brush_face(
		&actual.faces[0],
		&BrushFace {
			plane: x_plane(PlaneDir::Neg, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[1],
		&BrushFace {
			plane: y_plane(PlaneDir::Neg, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[2],
		&BrushFace {
			plane: z_plane(PlaneDir::Neg, -128.0),
			material_name: "C1A0_W2".to_owned(),
			material_axes: PlaneDir::Neg.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[3],
		&BrushFace {
			plane: z_plane(PlaneDir::Pos, 144.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_z(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[4],
		&BrushFace {
			plane: y_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_y(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
		},
	);

	check_brush_face(
		&actual.faces[5],
		&BrushFace {
			plane: x_plane(PlaneDir::Pos, 64.0),
			material_name: "C1A0_LABGLU".to_owned(),
			material_axes: PlaneDir::Pos.default_mat_axes_x(),
			material_offset: DVec2::new(0.0, 0.0),
			material_scale: DVec2::new(1.0, 1.0),
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

	pub fn default_mat_axes_x(&self) -> (DVec3, DVec3)
	{
		return match self
		{
			PlaneDir::Pos => (DVec3::new(0.0, 1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			PlaneDir::Neg => (DVec3::new(0.0, -1.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
		};
	}

	pub fn default_mat_axes_y(&self) -> (DVec3, DVec3)
	{
		return match self
		{
			PlaneDir::Pos => (DVec3::new(-1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
			PlaneDir::Neg => (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -1.0)),
		};
	}

	pub fn default_mat_axes_z(&self) -> (DVec3, DVec3)
	{
		return match self
		{
			PlaneDir::Pos => (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
			PlaneDir::Neg => (DVec3::new(-1.0, 0.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
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
