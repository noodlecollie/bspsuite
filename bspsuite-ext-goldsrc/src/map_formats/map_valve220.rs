use bspextifc::containers::map_blueprint_builder::{
	BuilderError, Entity, IMapBlueprintBuilder, MapBlueprintBuilder,
};
use bspextifc::types::{DPlane, DVec2, DVec3};
use glam;
use logos::Logos;
use std::ops::Range;

// Documentation on the Goldsrc map format:
// https://developer.valvesoftware.com/wiki/MAP_(file_format)
// In particular, the brush face format is defined as:
//   ( x1 y1 z1 ) ( x2 y2 z2 ) ( x3 y3 z3 ) TEXTURENAME [ Ux Uy Uz Uoffset ]
//   [ Vx Vy Vz Voffset ] rotation Uscale Vscale

// A helpful example of how to define Logos tokens:
// https://logos.maciej.codes/examples/json.html

type ParseResult = Result<(), ParseError>;

#[derive(Debug)]
struct ParseError
{
	token: Option<String>,
	location: Range<usize>,
	description: String,
}

impl ParseError
{
	// These constraints were confusing as heck. For how to establish them, see
	// https://users.rust-lang.org/t/constraining-an-associated-type-of-a-generic-parameter/136934
	pub fn from_token<'l, Ctx>(lexer: &logos::Lexer<'l, Ctx>, description: &str) -> Self
	where
		Ctx: logos::Logos<'l, Source: logos::Source<Slice<'l> = &'l str>>,
	{
		return Self {
			token: Some(lexer.slice().to_owned()),
			location: lexer.span(),
			description: description.to_owned(),
		};
	}

	pub fn from_span(location: Range<usize>, description: &str) -> Self
	{
		return Self {
			token: None,
			location: location,
			description: description.to_owned(),
		};
	}
}

enum BrushProgressionResult
{
	ParsedFace,
	FinishedBrush,
}

// When expecting a new entity (including worldspawn),
// declared with '{'.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
#[logos(error(Range<usize>, callback = |lex| lex.span()))]
enum BaseContext
{
	#[regex(r"//[^\n]*\n")]
	Comment,

	// Begins EntityContext
	#[token("{")]
	OpenBrace,
}

// When processing properties on an entity.
// Nested brushes are declared with '{'.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
#[logos(error(Range<usize>, callback = |lex| lex.span()))]
enum EntityContext
{
	#[regex(r"//[^\n]*\n")]
	Comment,

	// Begins BrushContext
	#[token("{")]
	OpenBrace,

	// Falls back to BaseContext
	#[token("}")]
	CloseBrace,

	// Borrowed from the JSON example. We can tweak this if it turns out not to be quite right.
	//          ( Non-term.     |  ( Escaped   | Unicode       ))
	#[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
	QuotedString(String),
}

// When processing brushes.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
#[logos(error(Range<usize>, callback = |lex| lex.span()))]
enum BrushContext
{
	#[regex(r"//[^\n]*\n")]
	Comment,

	// Falls back to EntityContext
	#[token("}")]
	CloseBrace,

	// Begins Point3DContext
	#[token("(")]
	OpenRoundBracket,

	// Begins VectorContext
	#[token("[")]
	OpenSquareBracket,

	// Borrowed from the JSON example.
	#[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
	Number(f64),

	// String covers any other chain of characters.
	// The string cannot begin with a character which
	// would match a different token. If we need something
	// more sophisticated than this, we may need to
	// create a new context to invoke at the correct time.
	#[regex(r"[^0-9\}\(\)\[\s-][^\s]*", |lex| lex.slice().to_owned())]
	String(String),
}

// When processing a 3D vector.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
#[logos(error(Range<usize>, callback = |lex| lex.span()))]
enum Point3DContext
{
	// Borrowed from the JSON example.
	#[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
	Number(f64),

	// Begins a new point.
	#[token("(")]
	OpenRoundBracket,

	// Ends a point. May fall back to BrushContext if this was the last point we expected.
	#[token(")")]
	CloseRoundBracket,
}

// When processing a vector of items.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
#[logos(error(Range<usize>, callback = |lex| lex.span()))]
enum VectorContext
{
	// Borrowed from the JSON example.
	#[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
	Number(f64),

	// Falls back to BrushContext.
	#[token("]")]
	CloseSquareBracket,
}

pub extern "C" fn parse()
{
	// TODO
}

fn parse_map(
	lexer: &mut logos::Lexer<'_, BaseContext>,
	builder: &mut MapBlueprintBuilder,
) -> ParseResult
{
	while let Some(token) = lexer.next()
	{
		match token
		{
			Ok(BaseContext::Comment) => continue,
			Ok(BaseContext::OpenBrace) =>
			{
				builder
					.begin_entity()
					.map_err(|_| ParseError::from_token(lexer, "begin_entity() failed"))?;

				let mut sub_lexer = lexer.clone().morph::<EntityContext>();
				parse_entity(&mut sub_lexer, builder)?;
				*lexer = sub_lexer.morph();

				builder
					.end_entity()
					.map_err(|_| ParseError::from_token(lexer, "end_entity() failed"))?;
			}
			Err(span) =>
			{
				return Err(ParseError::from_span(
					span,
					"Unrecognised token while parsing map",
				));
			}
		}
	}

	// Nothing more to parse, and we're in the correct state to finish parsing.
	return Ok(());
}

fn parse_entity(
	lexer: &mut logos::Lexer<'_, EntityContext>,
	builder: &mut MapBlueprintBuilder,
) -> ParseResult
{
	while let Some(token) = lexer.next()
	{
		match token
		{
			Ok(EntityContext::Comment) => continue,
			Ok(EntityContext::CloseBrace) => return Ok(()),
			Ok(EntityContext::QuotedString(key)) =>
			{
				parse_entity_value_after_key(lexer, builder, key)?
			}
			Ok(EntityContext::OpenBrace) =>
			{
				builder
					.begin_brush()
					.map_err(|_| ParseError::from_token(lexer, "begin_brush() failed"))?;

				let mut sub_lexer = lexer.clone().morph::<BrushContext>();
				parse_brush(&mut sub_lexer, builder)?;
				*lexer = sub_lexer.morph();

				builder
					.end_brush()
					.map_err(|_| ParseError::from_token(lexer, "end_brush() failed"))?;
			}
			Err(span) =>
			{
				return Err(ParseError::from_span(
					span,
					"Unrecognised token while parsing entity",
				));
			}
		}
	}

	return Err(ParseError::from_span(
		lexer.span(),
		"Unexpected end of input while parsing entity",
	));
}

fn parse_entity_value_after_key(
	lexer: &mut logos::Lexer<'_, EntityContext>,
	builder: &mut MapBlueprintBuilder,
	key: String,
) -> ParseResult
{
	return match lexer.next()
	{
		Some(token) => match token
		{
			Ok(EntityContext::QuotedString(value)) =>
			{
				builder
					.add_entity_keyvalue(&key, &value)
					.map_err(|_| ParseError::from_token(lexer, "add_entity_keyvalue() failed"))?;

				Ok(())
			}
			Ok(_) => Err(ParseError::from_token(
				lexer,
				"Unexpected token while parsing entity keyvalue property",
			)),
			Err(span) => Err(ParseError::from_span(
				span,
				"Unrecognised token while parsing entity keyvalue property",
			)),
		},
		None => Err(ParseError::from_span(
			lexer.span(),
			"Unexpected end of input while parsing entity keyvalue property",
		)),
	};
}

fn parse_brush(
	lexer: &mut logos::Lexer<'_, BrushContext>,
	builder: &mut MapBlueprintBuilder,
) -> ParseResult
{
	loop
	{
		let result = parse_brush_face_or_end_of_brush(lexer, builder)?;

		match result
		{
			BrushProgressionResult::ParsedFace => continue,
			BrushProgressionResult::FinishedBrush => break,
		};
	}

	return Ok(());
}

fn parse_brush_face_or_end_of_brush(
	lexer: &mut logos::Lexer<'_, BrushContext>,
	builder: &mut MapBlueprintBuilder,
) -> Result<BrushProgressionResult, ParseError>
{
	while let Some(token) = lexer.next()
	{
		match token
		{
			Ok(BrushContext::Comment) => continue,
			Ok(BrushContext::OpenRoundBracket) =>
			{
				parse_brush_face(lexer, builder)?;
				return Ok(BrushProgressionResult::ParsedFace);
			}
			Ok(BrushContext::CloseBrace) =>
			{
				return Ok(BrushProgressionResult::FinishedBrush);
			}
			Ok(_) =>
			{
				return Err(ParseError::from_token(
					lexer,
					"Unexpected token while parsing brush",
				));
			}
			Err(span) =>
			{
				return Err(ParseError::from_span(
					span,
					"Unrecognised token while parsing brush",
				));
			}
		}
	}

	// Make sure we consume the waiting token before we fail.
	lexer.next();

	return Err(ParseError::from_span(
		lexer.span(),
		"Unexpected end of input while parsing brush",
	));
}

fn parse_brush_face(
	lexer: &mut logos::Lexer<'_, BrushContext>,
	builder: &mut MapBlueprintBuilder,
) -> ParseResult
{
	let mut sub_lexer = lexer.clone().morph::<Point3DContext>();
	let plane_points: (DVec3, DVec3, DVec3) =
		parse_three_point3d_after_first_opening_bracket(&mut sub_lexer)?;
	*lexer = sub_lexer.morph();

	let material_path: String = parse_face_material_string(lexer)?;
	let u_axis_and_offset: (f64, f64, f64, f64) = parse_face_material_axis_and_offset(lexer)?;
	let v_axis_and_offset: (f64, f64, f64, f64) = parse_face_material_axis_and_offset(lexer)?;

	// We don't actually use the rotation, because we can derive it from the axes.
	let _: f64 = parse_face_material_number(lexer)?;

	let u_scale: f64 = parse_face_material_number(lexer)?;
	let v_scale: f64 = parse_face_material_number(lexer)?;

	let u_axis: DVec3 = DVec3::new(
		u_axis_and_offset.0,
		u_axis_and_offset.1,
		u_axis_and_offset.2,
	);
	let u_translation: f64 = u_axis_and_offset.3;
	let v_axis: DVec3 = DVec3::new(
		v_axis_and_offset.0,
		v_axis_and_offset.1,
		v_axis_and_offset.2,
	);
	let v_translation: f64 = v_axis_and_offset.3;

	builder
		.begin_brush_face()
		.map_err(|_| ParseError::from_token(lexer, "begin_brush_face() failed"))?;

	builder
		.set_brush_face_plane(plane_from_points(plane_points))
		.map_err(|_| ParseError::from_token(lexer, "set_brush_face_plane() failed"))?;

	builder
		.set_brush_face_material(material_path)
		.map_err(|_| ParseError::from_token(lexer, "set_brush_face_material() failed"))?;

	builder
		.set_brush_face_material_axes(u_axis, v_axis)
		.map_err(|_| ParseError::from_token(lexer, "set_brush_face_material_axes() failed"))?;

	builder
		.set_brush_face_material_scale(DVec2::new(u_scale, v_scale))
		.map_err(|_| ParseError::from_token(lexer, "set_brush_face_material_scale() failed"))?;

	builder
		.set_brush_face_material_translation(DVec2::new(u_translation, v_translation))
		.map_err(|_| {
			ParseError::from_token(lexer, "set_brush_face_material_translation() failed")
		})?;

	builder
		.end_brush_face()
		.map_err(|_| ParseError::from_token(lexer, "end_brush_face() failed"))?;

	return Ok(());
}

fn parse_three_point3d_after_first_opening_bracket(
	lexer: &mut logos::Lexer<'_, Point3DContext>,
) -> Result<(DVec3, DVec3, DVec3), ParseError>
{
	let mut points: [DVec3; 3] = [DVec3::NULL; 3];

	for iteration in 0..3
	{
		let point: DVec3 = parse_point3d_after_opening_bracket(lexer)?;

		if iteration < 2
		{
			// Ensure we have a following opening bracket.
			match lexer.next()
			{
				Some(token) => match token
				{
					Ok(Point3DContext::OpenRoundBracket) => Ok(()),
					Err(span) => Err(ParseError::from_span(
						span,
						"Unrecognised token while parsing 3D point",
					)),
					_ => Err(ParseError::from_token(
						lexer,
						"Unexpected token while parsing 3D point",
					)),
				},
				None => Err(ParseError::from_span(
					lexer.span(),
					"Unexpected end of input while parsing 3D point",
				)),
			}?;
		}

		points[iteration] = point;
	}

	return Ok((points[0], points[1], points[2]));
}

fn parse_point3d_after_opening_bracket(
	lexer: &mut logos::Lexer<'_, Point3DContext>,
) -> Result<DVec3, ParseError>
{
	let mut values: [f64; 3] = [0.0; 3];

	for iteration in 0..3
	{
		let value: f64 = match lexer.next()
		{
			Some(token) => match token
			{
				Ok(Point3DContext::Number(value)) => Ok(value),
				Ok(Point3DContext::CloseRoundBracket) => Err(ParseError::from_token(
					lexer,
					"Premature closing bracket when parsing 3D point",
				)),
				Err(span) => Err(ParseError::from_span(
					span,
					"Unrecognised token while parsing 3D point",
				)),
				_ => Err(ParseError::from_token(
					lexer,
					"Unexpected token while parsing 3D point",
				)),
			},
			None => Err(ParseError::from_span(
				lexer.span(),
				"Unexpected end of input while parsing 3D point",
			)),
		}?;

		values[iteration] = value;
	}

	// Ensure we have a closing bracket.
	match lexer.next()
	{
		Some(token) => match token
		{
			Ok(Point3DContext::CloseRoundBracket) => Ok(()),
			Err(span) => Err(ParseError::from_span(
				span,
				"Unrecognised token while parsing 3D point",
			)),
			_ => Err(ParseError::from_token(
				lexer,
				"Unexpected token while parsing 3D point",
			)),
		},
		None => Err(ParseError::from_span(
			lexer.span(),
			"Unexpected end of input while parsing 3D point",
		)),
	}?;

	return Ok(DVec3::new(values[0], values[1], values[2]));
}

fn parse_face_material_string(
	lexer: &mut logos::Lexer<'_, BrushContext>,
) -> Result<String, ParseError>
{
	return match lexer.next()
	{
		Some(token) => match token
		{
			Ok(BrushContext::String(value)) => Ok(value),
			Ok(_) => Err(ParseError::from_token(
				lexer,
				"Unexpected token while parsing face material string",
			)),
			Err(span) => Err(ParseError::from_span(
				span,
				"Unrecognised token while parsing face material string",
			)),
		},
		None => Err(ParseError::from_span(
			lexer.span(),
			"Unexpected end of input while parsing face material string",
		)),
	};
}

fn parse_face_material_axis_and_offset(
	lexer: &mut logos::Lexer<'_, BrushContext>,
) -> Result<(f64, f64, f64, f64), ParseError>
{
	return match lexer.next()
	{
		Some(token) => match token
		{
			Ok(BrushContext::OpenSquareBracket) =>
			{
				let mut sub_lexer = lexer.clone().morph::<VectorContext>();
				let vals: [f64; 4] =
					parse_numeric_vector_after_opening_bracket::<4>(&mut sub_lexer)?;
				*lexer = sub_lexer.morph();

				Ok((vals[0], vals[1], vals[2], vals[3]))
			}
			Ok(_) => Err(ParseError::from_token(
				lexer,
				"Unexpected token when opening bracket of numeric vector was expected",
			)),
			Err(span) => Err(ParseError::from_span(
				span,
				"Unrecognised token when opening bracket of numeric vector was expected",
			)),
		},
		None => Err(ParseError::from_span(
			lexer.span(),
			"Unexpected token when opening bracket of numeric vector was expected",
		)),
	};
}

fn parse_numeric_vector_after_opening_bracket<const LENGTH: usize>(
	lexer: &mut logos::Lexer<'_, VectorContext>,
) -> Result<[f64; LENGTH], ParseError>
{
	let mut values: [f64; LENGTH] = [0.0; LENGTH];

	for iteration in 0..LENGTH
	{
		let value: f64 = match lexer.next()
		{
			Some(token) => match token
			{
				Ok(VectorContext::Number(val)) => Ok(val),
				Ok(_) => Err(ParseError::from_token(
					lexer,
					"Unexpected token while parsing numeric vector",
				)),
				Err(span) => Err(ParseError::from_span(
					span,
					"Unrecognised token while parsing numeric vector",
				)),
			},
			None => Err(ParseError::from_span(
				lexer.span(),
				"Unexpected end of input while parsing numeric vector",
			)),
		}?;

		values[iteration] = value;
	}

	match lexer.next()
	{
		Some(token) => match token
		{
			Ok(VectorContext::CloseSquareBracket) => (),
			Ok(_) =>
			{
				return Err(ParseError::from_token(
					lexer,
					"Unexpected token when closing bracket of numeric vector was expected",
				));
			}
			Err(span) =>
			{
				return Err(ParseError::from_span(
					span,
					"Unrecognised token when closing bracket of numeric vector was expected",
				));
			}
		},
		None =>
		{
			return Err(ParseError::from_span(
				lexer.span(),
				"Unexpected end of input when closing bracket of numeric vector was expected",
			));
		}
	};

	return Ok(values);
}

fn parse_face_material_number(lexer: &mut logos::Lexer<'_, BrushContext>)
-> Result<f64, ParseError>
{
	return match lexer.next()
	{
		Some(token) => match token
		{
			Ok(BrushContext::Number(val)) => Ok(val),
			Ok(_) => Err(ParseError::from_token(
				lexer,
				"Unexpected token while parsing numeric value",
			)),
			Err(span) => Err(ParseError::from_span(
				span,
				"Unrecognised token while parsing numeric value",
			)),
		},
		None => Err(ParseError::from_span(
			lexer.span(),
			"Unexpected end of input while parsing numeric value",
		)),
	};
}

// Based on https://stackoverflow.com/a/53698872
fn plane_from_points(points: (DVec3, DVec3, DVec3)) -> DPlane
{
	type GVec3 = glam::DVec3;

	let p0: GVec3 = GVec3::new(points.0.x, points.0.y, points.0.z);
	let p1: GVec3 = GVec3::new(points.1.x, points.1.y, points.1.z);
	let p2: GVec3 = GVec3::new(points.2.x, points.2.y, points.2.z);

	let u: GVec3 = p1 - p0;
	let v: GVec3 = p2 - p0;
	let normal: GVec3 = u.cross(v).try_normalize().unwrap_or_else(|| GVec3::ZERO);
	let distance: f64 = p0.dot(normal);

	return DPlane::new(DVec3::new(normal.x, normal.y, normal.z), distance);
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::map_formats::test_resources::goldsrcmap_res::BOX_MAP_SOURCE;

	#[test]
	fn parse_box_map()
	{
		let mut lexer: logos::Lexer<'_, BaseContext> = BaseContext::lexer(BOX_MAP_SOURCE);
		let mut builder: MapBlueprintBuilder = MapBlueprintBuilder::new();
		let result = parse_map(&mut lexer, &mut builder);
		let build_result: Result<Vec<Entity>, BuilderError> = builder.collect();

		assert!(result.is_ok());
		assert!(build_result.is_ok());
	}
}
