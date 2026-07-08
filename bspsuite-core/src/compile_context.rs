use crate::commands::{CompileArgs, InputPathMetadata};
use crate::compiler_error::{CompilerError, CompilerErrorCode};
use crate::configs::{CompileTuningParametersConfig, GameConfig};
use crate::io::{CompileTuningParametersConfigFileIO, VersionedIOFormat};
use crate::math::CompileTuningParameters;
use crate::toolchain::Toolchain;
use anyhow::Result;

pub struct CompileContext
{
	pub toolchain: Toolchain,
	pub game_config: GameConfig,
	pub input_path_metadata: InputPathMetadata,
	pub compile_tuning_parameters: CompileTuningParameters,
	pub map_format_override: Option<String>,
}

impl CompileContext
{
	pub fn create(args: &CompileArgs) -> Result<Self, CompilerError>
	{
		let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root);

		// TODO: Make load_for_game() return compiler error?
		let game_config: GameConfig =
			GameConfig::load_for_game(toolchain.root_path(), args.game.as_str())
				.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ConfigError, err))?;

		let parameters_file: Option<CompileTuningParametersConfig> = if let Some(path_buf) =
			args.parameters_file.as_ref()
		{
			Some(
				CompileTuningParametersConfigFileIO::read(path_buf.as_path()).map_err(|err| {
					CompilerError::from_anyhow(CompilerErrorCode::ConfigError, err)
				})?,
			)
		}
		else
		{
			None
		};

		let mut parameters_config: CompileTuningParametersConfig = game_config
			.default_compile_tuning_parameters
			.as_ref()
			.map(|params| params.clone())
			.unwrap_or_default();

		if let Some(override_parameters) = parameters_file
		{
			parameters_config =
				CompileTuningParametersConfig::merge(parameters_config, override_parameters);
		}

		let input_path_md: InputPathMetadata = InputPathMetadata::new(&args.input_file)
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err))?;

		return Ok(Self {
			toolchain: toolchain,
			game_config: game_config,
			input_path_metadata: input_path_md,
			compile_tuning_parameters: create_compile_tuning_parameters(parameters_config),
			map_format_override: args
				.map_format_override
				.as_ref()
				.map(|str_ref| str_ref.to_string()),
		});
	}
}

fn create_compile_tuning_parameters(
	config: CompileTuningParametersConfig,
) -> CompileTuningParameters
{
	let defaults: CompileTuningParameters = CompileTuningParameters::default();

	return CompileTuningParameters {
		contact_epsilon: config.contact_epsilon.unwrap_or(defaults.contact_epsilon),
		equal_point_radius_epsilon: config
			.equal_point_radius_epsilon
			.unwrap_or(defaults.equal_point_radius_epsilon),
		equal_vector_component_epsilon: config
			.equal_vector_component_epsilon
			.unwrap_or(defaults.equal_vector_component_epsilon),
		zero_epsilon: config.zero_epsilon.unwrap_or(defaults.zero_epsilon),
	};
}
