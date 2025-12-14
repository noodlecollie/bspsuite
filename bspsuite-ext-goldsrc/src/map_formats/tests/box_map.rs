use crate::map_formats::map_valve220::parse_map;
use bspextifc::containers::map_blueprint_builder::{
	Brush, BuilderError, Entity, MapBlueprintBuilder,
};

const BOX_MAP_SOURCE: &str = r#"
// entity 0
{
	"classname" "worldspawn"
	"mapversion" "510"
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
		Some("510".to_owned()).as_ref()
	);

	// TODO: Check entity 1

	let brushes: &Vec<Brush> = &world.brushes;
	assert_eq!(brushes.len(), 6);

	// TODO: Check each brush
}
