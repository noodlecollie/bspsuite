use std::path::PathBuf;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_residual_errors;
use crate::compile_context::CompileContext;
use crate::compiler_error::{CompilerError, CompilerErrorCode};
use crate::extensions::{ExtensionList, extension_routines};
use crate::io::{MapGeomFileIO, MapSourceFileIO, VersionedIOFormat};
use crate::model::{MapGeomFile, MapSourceFile};
use anyhow::{Context, Result, anyhow};
use bspextifc::builders::map_source_builder::{Entity, MapSourceBuilder};
use log::{debug, info};

pub struct CompileArgs
{
	pub base: BaseArgs,
	pub input_file: PathBuf,
	pub game: String,
	pub map_format_override: Option<String>,
	pub parameters_file: Option<PathBuf>,
	pub dump_source_file: bool,
}

pub fn bspcore_run_compile(args: &CompileArgs) -> ResultCode
{
	return wrap_residual_errors(|| run_compile(args));
}

fn run_compile(args: &CompileArgs) -> Result<(), CompilerError>
{
	let ctx: CompileContext = CompileContext::create(args)?;

	debug!(
		"Allowed map formats for game {}: {}",
		ctx.game_config.game_id,
		ctx.game_config.map_formats.join(", ")
	);

	let extensions: ExtensionList = ctx.toolchain.find_extensions();
	extension_routines::register_map_formats(&extensions);

	let map_format_result: Result<extension_routines::ExtensionForParsingMapFormat> =
		extension_routines::choose_extension_to_parse_map(
			&extensions,
			ctx.input_path_metadata.full_path.as_path(),
			&ctx.game_config
				.map_formats
				.iter()
				.map(|fmt| fmt.as_str())
				.collect(),
			&ctx.map_format_override.as_ref().map(|s| s.as_str()),
		);

	let map_format: extension_routines::ExtensionForParsingMapFormat = map_format_result
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err))?;

	debug!(
		"Input map format: {} ({})",
		map_format.map_format_name,
		ctx.map_format_override
			.map(|_| "provided as argument")
			.unwrap_or("inferred from file extension")
	);

	info!("Loading {}", ctx.input_path_metadata.full_path.display());

	let input_file: String = std::fs::read_to_string(&ctx.input_path_metadata.full_path)
		.with_context(|| {
			format!(
				"Failed to read {}",
				ctx.input_path_metadata.full_path.display()
			)
		})
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	info!("Parsing {}", ctx.input_path_metadata.full_path.display());

	let builder: MapSourceBuilder =
		extension_routines::parse_map(&extensions, &input_file, &map_format)
			.with_context(|| "Failed to initiate map parsing")
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::InternalError, err))?;

	let parsed_entities: Vec<Entity> = builder.collect().map_err(|err| {
		CompilerError::from_anyhow(
			CompilerErrorCode::IoError,
			anyhow!(
				"Failed to parse map {}. {err}",
				ctx.input_path_metadata.full_path.display()
			),
		)
	})?;

	let map_source: MapSourceFile =
		MapSourceFile::create(parsed_entities, &ctx.compile_tuning_parameters);

	debug!(
		"Input map parsed successfully, contains {} entities",
		map_source.entities.len()
	);

	MapSourceFileIO::write(
		ctx.input_path_metadata
			.directory_path
			.join(format!("{}.source.json", ctx.input_path_metadata.file_name))
			.as_path(),
		&map_source,
	)
	.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	let map_geometry: MapGeomFile =
		MapGeomFile::construct(map_source, &ctx.compile_tuning_parameters)
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::InternalError, err))?;

	MapGeomFileIO::write(
		ctx.input_path_metadata
			.directory_path
			.join(format!(
				"{}.geometry.json",
				ctx.input_path_metadata.file_name
			))
			.as_path(),
		&map_geometry,
	)
	.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	info!("Compile complete");

	return Ok(());
}
