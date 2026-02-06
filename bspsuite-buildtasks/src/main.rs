// Let's try and keep this build as clean as possible.
#![deny(unused_variables)]
#![deny(dead_code)]
#![deny(unused_imports)]

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{Context, Error, bail, ensure};
use clap::Parser;
use glob;
use glob::Paths;
use paris::LogIcon;
use target_lexicon::{HOST, OperatingSystem};

// A lot of code in this file is based off
// https://github.com/matklad/cargo-xtask/blob/master/examples/hello-world/xtask/src/main.rs

/// Commands that may be executed on the compiler executable.
#[derive(clap::Parser)]
#[command(version, about = "BSPSuite build task runner", long_about = None)]
pub struct Cli
{
	#[command(subcommand)]
	pub command: Subcommand,
}

#[derive(clap::Subcommand, strum::Display)]
pub enum Subcommand
{
	/// Build the compiler toolchain and copy it
	/// into a canonical directory structure.
	Build,
}

fn main()
{
	let parsed_args: Cli = Cli::parse();
	let subcommand: &Subcommand = &parsed_args.command;
	let result: Result<(), Error> = match subcommand
	{
		Subcommand::Build => run_build_command(),
	};

	if let Err(e) = result
	{
		for item in e.chain()
		{
			eprintln!("{item}");
		}
	};
}

fn run_build_command() -> Result<(), Error>
{
	// Compiler also builds core library.
	build_crate("bspsuite-compiler")?;

	build_extensions()?;

	// Utilities (TODO: Make these optional)
	build_crate("bspsuite-visualiser")?;

	let src_dir: PathBuf = binaries_dir();
	let dist_dir: PathBuf = src_dir.join("dist");

	create_dist_dir(&dist_dir)?;

	let lib_prefix: &str = library_prefix_for_platform();
	let lib_ext: &str = library_extension_for_platform();
	let exe_ext: &str = executable_extension_for_platform();

	copy_named_file(&src_dir, &dist_dir, format!("bspc{exe_ext}").as_str())?;
	copy_named_file(
		&src_dir,
		&dist_dir,
		format!("{lib_prefix}bspcore{lib_ext}").as_str(),
	)?;

	let glob_str: String = format!("{lib_prefix}*ext{lib_ext}");
	copy_glob(&src_dir, &dist_dir.join("extensions"), glob_str.as_str())?;

	copy_extension_game_configs(&dist_dir)?;

	// Optional utilities
	copy_named_file_optional(&src_dir, &dist_dir, format!("bspviz{exe_ext}").as_str())?;

	Ok(())
}

fn copy_extension_game_configs(dist_dir: &PathBuf) -> Result<(), Error>
{
	let mut copied: HashMap<PathBuf, String> = HashMap::new();
	return for_each_extension_project(|path| copy_game_configs(path, dist_dir, &mut copied));
}

fn build_extensions() -> Result<(), Error>
{
	return for_each_extension_project(|path| build_crate(path.to_str().unwrap()));
}

fn build_crate(dir_name: &str) -> Result<(), Error>
{
	let result = run_cargo(&["build"], &project_root().join(dir_name))?;
	ensure!(result.success(), "Failed to build {dir_name}");
	Ok(())
}

fn copy_game_configs(
	source_dir: &Path,
	dist_dir: &PathBuf,
	already_copied: &mut HashMap<PathBuf, String>,
) -> Result<(), Error>
{
	let ext_dirname: &str = source_dir
		.parent()
		.unwrap()
		.file_name()
		.unwrap()
		.to_str()
		.unwrap();

	if !source_dir.join("games").is_dir()
	{
		return Ok(());
	}

	let glob_str: String = format!("{}/games/*/config.toml", source_dir.to_str().unwrap());
	let glob_result: Paths = glob::glob(glob_str.as_str())?;

	for file in glob_result
	{
		let file_rel_path: PathBuf = file
			.as_ref()
			.unwrap()
			.strip_prefix(&source_dir)
			.map(|val| val.to_path_buf())?;

		if let Some(orig_ext) = already_copied.get(&file_rel_path)
		{
			bail!(
				"Extension {ext_dirname} provides game config file {} that conflicts \
				with a config of the same name from extension {orig_ext}",
				file_rel_path.to_str().unwrap()
			);
		}

		copy_relative_file(&source_dir.to_path_buf(), &file_rel_path, dist_dir, true)?;
		already_copied.insert(file_rel_path, ext_dirname.to_owned());
	}

	return Ok(());
}

fn run_cargo(args: &[&str], cwd: &PathBuf) -> Result<ExitStatus, std::io::Error>
{
	let cargo: String = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
	return Command::new(cargo).current_dir(cwd).args(args).status();
}

fn create_dist_dir(dist_dir: &PathBuf) -> Result<(), Error>
{
	create_dir(&dist_dir)?;
	create_dir(&dist_dir.join("games"))?;
	create_dir(&dist_dir.join("extensions"))?;

	Ok(())
}

fn create_dir(dir: &PathBuf) -> Result<(), Error>
{
	ensure!(
		!dir.is_file(),
		"Failed to create directory: {} is actually a file",
		dir.to_str().unwrap()
	);

	if !dir.is_dir()
	{
		std::fs::create_dir(dir.clone())
			.with_context(|| format!("Failed to create directory {}", dir.to_str().unwrap()))?;
	}

	Ok(())
}

fn copy_glob(src: &PathBuf, dest: &PathBuf, glob_str: &str) -> Result<(), Error>
{
	let full_glob_str: String = format!("{}/{glob_str}", src.to_str().unwrap());
	let glob_result: Paths = glob::glob(full_glob_str.as_str())?;

	for source_file in glob_result
	{
		copy_named_file(
			src,
			dest,
			source_file.unwrap().file_name().unwrap().to_str().unwrap(),
		)?;
	}

	Ok(())
}

fn copy_relative_file(
	src_root: &PathBuf,
	src_rel: &PathBuf,
	dest_root: &PathBuf,
	create_dirs: bool,
) -> Result<(), Error>
{
	let dest_full_path: PathBuf = dest_root.join(src_rel);

	if create_dirs
	{
		let dest_dir: &Path = dest_full_path
			.parent()
			.expect("Could not get directory name of destination file");

		std::fs::create_dir_all(dest_dir)?;
	}

	return copy_file(&src_root.join(src_rel), &dest_full_path);
}

fn copy_named_file_optional(src: &PathBuf, dest: &PathBuf, name: &str) -> Result<(), Error>
{
	let source_path: PathBuf = src.join(name);

	if source_path.is_file()
	{
		return copy_file(&src.join(name), &dest.join(name));
	}

	return Ok(());
}

fn copy_named_file(src: &PathBuf, dest: &PathBuf, name: &str) -> Result<(), Error>
{
	return copy_file(&src.join(name), &dest.join(name));
}

fn copy_file(source_path: &PathBuf, dest_path: &PathBuf) -> Result<(), Error>
{
	if source_path == dest_path
	{
		ensure!(
			source_path.exists(),
			"File {} does not exist",
			source_path.to_str().unwrap()
		);

		return Ok(());
	}

	let should_copy = {
		if !dest_path.is_file()
		{
			true
		}
		else
		{
			let source_metadata = source_path.metadata().with_context(|| {
				format!(
					"Failed to get file metadata for {}",
					source_path.to_str().unwrap()
				)
			})?;

			let dest_metadata = dest_path.metadata().with_context(|| {
				format!(
					"Failed to get file metadata for {}",
					dest_path.to_str().unwrap()
				)
			})?;

			source_metadata.modified()? > dest_metadata.modified()?
		}
	};

	if should_copy
	{
		std::fs::copy(&source_path, &dest_path)
			.with_context(|| format!("Failed to copy {}", source_path.to_str().unwrap()))?;

		println!(
			"{} {} -> {}",
			LogIcon::Tick,
			source_path.to_str().unwrap(),
			dest_path.to_str().unwrap()
		);
	}
	else
	{
		println!("• {}", dest_path.to_str().unwrap())
	}

	Ok(())
}

fn for_each_extension_project<Callback>(mut callback: Callback) -> Result<(), Error>
where
	Callback: FnMut(&Path) -> Result<(), Error>,
{
	let glob_str: String = format!("{}/bspsuite-ext-*", project_root().to_str().unwrap());
	let glob_result: Paths = glob::glob(glob_str.as_str()).unwrap();

	for path in glob_result
	{
		if let Ok(path) = path
		{
			if path.is_dir()
			{
				callback(path.as_path())?;
			}
		}
	}

	Ok(())
}

fn project_root() -> PathBuf
{
	Path::new(&env!("CARGO_MANIFEST_DIR"))
		.ancestors()
		.nth(1)
		.unwrap()
		.to_path_buf()
}

fn binaries_dir() -> PathBuf
{
	Path::new(&env!("OUT_DIR"))
		.ancestors()
		.nth(3)
		.unwrap()
		.to_path_buf()
}

const fn library_extension_for_platform() -> &'static str
{
	return match HOST.operating_system
	{
		OperatingSystem::Windows => ".dll",
		OperatingSystem::Linux => ".so",
		_ => panic!("Unsupported operating system"),
	};
}

const fn library_prefix_for_platform() -> &'static str
{
	return match HOST.operating_system
	{
		OperatingSystem::Windows => "",
		OperatingSystem::Linux => "lib",
		_ => panic!("Unsupported operating system"),
	};
}

const fn executable_extension_for_platform() -> &'static str
{
	return match HOST.operating_system
	{
		OperatingSystem::Windows => ".exe",
		OperatingSystem::Linux => "",
		_ => panic!("Unsupported operating system"),
	};
}
