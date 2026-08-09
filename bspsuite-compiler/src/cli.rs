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
	/// Print information about available extensions.
	Extinfo(ExtinfoCommandArgs),

	/// Compile a map from a source file.
	Compile(CompileCommandArgs),

	/// Print information about a particular game resource.
	Resinfo(ResinfoCommandArgs),
}

#[derive(clap::Args)]
pub struct CompileCommandArgs
{
	/// Path to map source file that will be compiled.
	#[arg()]
	pub input_file: PathBuf,

	/// Name of the game to compile for. Should correspond to a subdirectory
	/// under the compiler toolchain's 'games' directory.
	#[arg(short, long)]
	pub game: String,

	/// Directory on disk for the target game.
	#[arg(short('d'), long)]
	pub game_dir: PathBuf,

	/// Format of the map source file. If this is not specified, the compiler
	/// will attempt to infer the format based on the file extension and the
	/// target game.
	#[arg(short('f'), long)]
	pub map_format: Option<String>,

	/// Optional TOML file containing compile tuning parameters that will
	/// override the defaults set in the game config file.
	#[arg(long)]
	pub parameters_file: Option<PathBuf>,

	/// If set, an internal representation of the parsed map source file is
	/// dumped to the input directory.
	#[arg(long)]
	pub dump_source_file: bool,
}

#[derive(clap::Args)]
pub struct ExtinfoCommandArgs
{
	/// Extension to fetch information about. If omitted, lists available
	/// extensions.
	#[arg()]
	pub extension: Option<String>,
}

#[derive(clap::Args)]
pub struct ResinfoCommandArgs
{
	/// Name of the game to use. Should correspond to a subdirectory
	/// under the compiler toolchain's 'games' directory.
	#[arg(short, long)]
	pub game: String,

	/// Resource filesystem roots. These can be directories or package files,
	/// depending on what the game supports.
	#[arg(short, long, required = true)]
	pub vfs_roots: Vec<String>,

	/// Resource path to display information about.
	#[arg()]
	pub resource_path: String,
}
