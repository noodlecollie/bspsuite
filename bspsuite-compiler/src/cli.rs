use std::path::PathBuf;

#[derive(Copy, Clone, Debug, strum::Display, clap::ValueEnum)]
pub enum DebugLevel
{
	/// No debug logging will occur.
	Off,

	/// Normal debug logs will be printed.
	On,

	/// Debug and trace logs will be printed.
	Trace,
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None, display_name = env!("CARGO_BIN_NAME"))]
pub struct Cli
{
	#[arg(short, long)]
	pub debug: Option<DebugLevel>,

	/// Root directory under which to look for BSPSuite configs and
	/// extensions. If not specified, defaults to the application directory.
	#[arg(long)]
	pub toolchain_root: Option<PathBuf>,

	#[command(subcommand)]
	pub command: Subcommand,
}

#[derive(clap::Subcommand, strum::Display)]
pub enum Subcommand
{
	/// Print information about the compiler and available extensions.
	Info(InfoCommandArgs),

	/// Compile a map from a source file.
	Compile(CompileCommandArgs),
}

#[derive(clap::Args)]
pub struct CompileCommandArgs
{
	/// Path to map source file that will be compiled.
	#[arg()]
	pub input_file: PathBuf,
}

#[derive(clap::Args)]
pub struct InfoCommandArgs
{
	/// Log information about supported map formats.
	#[arg(long)]
	pub map_formats: bool,
}
