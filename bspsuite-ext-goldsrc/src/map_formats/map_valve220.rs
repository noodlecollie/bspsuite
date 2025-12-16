use bspextifc::builders::map_blueprint_builder::{IMapBlueprintBuilder, MapBlueprintBuilder};
use bspextifc::types::{DPlane, DVec2, DVec3, LineCounter, ParseError, ParseResult};
use glam;
use logos::Logos;

// Documentation on the Goldsrc map format:
// https://developer.valvesoftware.com/wiki/MAP_(file_format)
// Also see notes/MAPFiles_2001_StefanHajnoczi.pdf in this repo.

// The brush face format is defined as:
//   ( x1 y1 z1 ) ( x2 y2 z2 ) ( x3 y3 z3 ) TEXTURENAME [ Ux Uy Uz Uoffset ]
//   [ Vx Vy Vz Voffset ] rotation Uscale Vscale

// A helpful example of how to define Logos tokens:
// https://logos.maciej.codes/examples/json.html

enum BrushProgressionResult
{
	ParsedFace,
	FinishedBrush,
}

// When expecting a new entity (including worldspawn),
// declared with '{'.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = LineCounter)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(skip(r"\n", update_line_count))]
enum BaseContext
{
	#[regex(r"//[^\n]*")]
	Comment,

	// Begins EntityContext
	#[token("{")]
	OpenBrace,
}

// When processing properties on an entity.
// Nested brushes are declared with '{'.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = LineCounter)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(skip(r"\n", update_line_count))]
enum EntityContext
{
	#[regex(r"//[^\n]*")]
	Comment,

	// Begins BrushContext
	#[token("{")]
	OpenBrace,

	// Falls back to BaseContext
	#[token("}")]
	CloseBrace,

	// Borrowed from the JSON example. We can tweak this if it turns out not to be quite right.
	// We know that this token will include a leading and trailing quote, so we strip them both.
	//          ( Non-term.     |  ( Escaped   | Unicode       ))
	#[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| remove_leading_and_trailing_char(lex.slice()))]
	QuotedString(String),
}

// When processing brushes.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = LineCounter)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(skip(r"\n", update_line_count))]
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
#[logos(extras = LineCounter)]
#[logos(skip r"[ \t]+")]
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
#[logos(extras = LineCounter)]
#[logos(skip r"[ \t]+")]
enum VectorContext
{
	// Borrowed from the JSON example.
	#[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
	Number(f64),

	// Falls back to BrushContext.
	#[token("]")]
	CloseSquareBracket,
}

// When parsing a material string.
// This has to be different from the brush context, since
// texture names can start with '{' in GoldSrc to indicate
// that they're masked, and otherwise this would be confused
// with opening a new brush.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = LineCounter)]
#[logos(skip r"[ \t]+")]
enum MaterialNameContext
{
	#[regex(r"\S+", |lex| lex.slice().to_owned())]
	String(String),
}

pub extern "C" fn parse()
{
	// TODO
}

pub fn parse_map(source: &str, builder: &mut MapBlueprintBuilder) -> ParseResult
{
	let mut lexer: logos::Lexer<'_, BaseContext> = BaseContext::lexer(source);

	while let Some(token) = lexer.next()
	{
		match token
		{
			Ok(BaseContext::Comment) => continue,
			Ok(BaseContext::OpenBrace) =>
			{
				builder
					.begin_entity()
					.map_err(|_| parse_error(&lexer, "begin_entity() failed"))?;

				let mut sub_lexer = lexer.clone().morph::<EntityContext>();
				parse_entity(&mut sub_lexer, builder)?;
				lexer = sub_lexer.morph();

				builder
					.end_entity()
					.map_err(|_| parse_error(&lexer, "end_entity() failed"))?;
			}
			Err(()) =>
			{
				return Err(parse_error(&lexer, "Unrecognised token while parsing map"));
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
					.map_err(|_| parse_error(lexer, "begin_brush() failed"))?;

				let mut sub_lexer = lexer.clone().morph::<BrushContext>();
				parse_brush(&mut sub_lexer, builder)?;
				*lexer = sub_lexer.morph();

				builder
					.end_brush()
					.map_err(|_| parse_error(lexer, "end_brush() failed"))?;
			}
			Err(()) =>
			{
				return Err(parse_error(
					lexer,
					"Unrecognised token while parsing entity",
				));
			}
		}
	}

	return Err(parse_error(
		lexer,
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
					.map_err(|_| parse_error(lexer, "add_entity_keyvalue() failed"))?;

				Ok(())
			}
			Ok(_) => Err(parse_error(
				lexer,
				"Unexpected token while parsing entity keyvalue property",
			)),
			Err(()) => Err(parse_error(
				lexer,
				"Unrecognised token while parsing entity keyvalue property",
			)),
		},
		None => Err(parse_error(
			lexer,
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
				return Err(parse_error(lexer, "Unexpected token while parsing brush"));
			}
			Err(()) =>
			{
				return Err(parse_error(lexer, "Unrecognised token while parsing brush"));
			}
		}
	}

	// Make sure we consume the waiting token before we fail.
	lexer.next();

	return Err(parse_error(
		lexer,
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

	let mut sub_lexer = lexer.clone().morph::<MaterialNameContext>();
	let material_path: String = parse_face_material_string(&mut sub_lexer)?;
	*lexer = sub_lexer.morph();

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
		.map_err(|_| parse_error(lexer, "begin_brush_face() failed"))?;

	builder
		.set_brush_face_plane(plane_from_points(plane_points))
		.map_err(|_| parse_error(lexer, "set_brush_face_plane() failed"))?;

	builder
		.set_brush_face_material(material_path)
		.map_err(|_| parse_error(lexer, "set_brush_face_material() failed"))?;

	builder
		.set_brush_face_material_axes(u_axis, v_axis)
		.map_err(|_| parse_error(lexer, "set_brush_face_material_axes() failed"))?;

	builder
		.set_brush_face_material_scale(DVec2::new(u_scale, v_scale))
		.map_err(|_| parse_error(lexer, "set_brush_face_material_scale() failed"))?;

	builder
		.set_brush_face_material_translation(DVec2::new(u_translation, v_translation))
		.map_err(|_| parse_error(lexer, "set_brush_face_material_translation() failed"))?;

	builder
		.end_brush_face()
		.map_err(|_| parse_error(lexer, "end_brush_face() failed"))?;

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
					Err(()) => Err(parse_error(
						lexer,
						"Unrecognised token while parsing 3D point",
					)),
					_ => Err(parse_error(
						lexer,
						"Unexpected token while parsing 3D point",
					)),
				},
				None => Err(parse_error(
					lexer,
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
				Ok(Point3DContext::CloseRoundBracket) => Err(parse_error(
					lexer,
					"Premature closing bracket when parsing 3D point",
				)),
				Err(()) => Err(parse_error(
					lexer,
					"Unrecognised token while parsing 3D point",
				)),
				_ => Err(parse_error(
					lexer,
					"Unexpected token while parsing 3D point",
				)),
			},
			None => Err(parse_error(
				lexer,
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
			Err(()) => Err(parse_error(
				lexer,
				"Unrecognised token while parsing 3D point",
			)),
			_ => Err(parse_error(
				lexer,
				"Unexpected token while parsing 3D point",
			)),
		},
		None => Err(parse_error(
			lexer,
			"Unexpected end of input while parsing 3D point",
		)),
	}?;

	return Ok(DVec3::new(values[0], values[1], values[2]));
}

fn parse_face_material_string(
	lexer: &mut logos::Lexer<'_, MaterialNameContext>,
) -> Result<String, ParseError>
{
	return match lexer.next()
	{
		Some(token) => match token
		{
			Ok(MaterialNameContext::String(value)) => Ok(value),
			Err(()) => Err(parse_error(
				lexer,
				"Unrecognised token while parsing face material string",
			)),
		},
		None => Err(parse_error(
			lexer,
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
			Ok(_) => Err(parse_error(
				lexer,
				"Unexpected token when opening bracket of numeric vector was expected",
			)),
			Err(()) => Err(parse_error(
				lexer,
				"Unrecognised token when opening bracket of numeric vector was expected",
			)),
		},
		None => Err(parse_error(
			lexer,
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
				Ok(_) => Err(parse_error(
					lexer,
					"Unexpected token while parsing numeric vector",
				)),
				Err(()) => Err(parse_error(
					lexer,
					"Unrecognised token while parsing numeric vector",
				)),
			},
			None => Err(parse_error(
				lexer,
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
				return Err(parse_error(
					lexer,
					"Unexpected token when closing bracket of numeric vector was expected",
				));
			}
			Err(()) =>
			{
				return Err(parse_error(
					lexer,
					"Unrecognised token when closing bracket of numeric vector was expected",
				));
			}
		},
		None =>
		{
			return Err(parse_error(
				lexer,
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
			Ok(_) => Err(parse_error(
				lexer,
				"Unexpected token while parsing numeric value",
			)),
			Err(()) => Err(parse_error(
				lexer,
				"Unrecognised token while parsing numeric value",
			)),
		},
		None => Err(parse_error(
			lexer,
			"Unexpected end of input while parsing numeric value",
		)),
	};
}

// Based on https://github.com/stefanha/map-files/blob/master/math.h#L177
// For some reason (glam handedness?), the cross product order must be
// inverted from the original reference code to produce planes with the
// orientation that we expect. This was found by trial and error.
fn plane_from_points(points: (DVec3, DVec3, DVec3)) -> DPlane
{
	type GVec3 = glam::DVec3;

	let a: GVec3 = GVec3::new(points.0.x, points.0.y, points.0.z);
	let b: GVec3 = GVec3::new(points.1.x, points.1.y, points.1.z);
	let c: GVec3 = GVec3::new(points.2.x, points.2.y, points.2.z);

	let b_to_c: GVec3 = c - b;
	let b_to_a: GVec3 = a - b;

	let normal: GVec3 = b_to_a
		.cross(b_to_c)
		.try_normalize()
		.unwrap_or_else(|| GVec3::ZERO);

	let distance: f64 = normal.dot(a);

	return DPlane::new(DVec3::new(normal.x, normal.y, normal.z), distance);
}

fn update_line_count<'l, Ctx>(lexer: &mut logos::Lexer<'l, Ctx>) -> logos::Skip
where
	Ctx: logos::Logos<'l, Extras = LineCounter>,
{
	lexer.extras.record_new_line(lexer.span().end);
	return logos::Skip;
}

// These constraints were confusing as heck. For how to establish them, see
// https://users.rust-lang.org/t/constraining-an-associated-type-of-a-generic-parameter/136934
fn parse_error<'l, Ctx>(lexer: &logos::Lexer<'l, Ctx>, description: &str) -> ParseError
where
	Ctx: logos::Logos<'l, Extras = LineCounter, Source: logos::Source<Slice<'l> = &'l str>>,
{
	let location: (usize, usize) = lexer.extras.get_line_and_column(lexer.span());
	let slice: &str = lexer.slice();
	let token: Option<String> = if !slice.is_empty()
	{
		Some(slice.to_owned())
	}
	else
	{
		None
	};

	return ParseError::new(location.0, location.1, token, Some(description.to_owned()));
}

fn remove_leading_and_trailing_char(token: &str) -> String
{
	return match token.len()
	{
		0 | 1 => "".to_owned(),
		_ => token[1..token.len() - 1].to_owned(),
	};
}
