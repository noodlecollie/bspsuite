use std::collections::HashMap;
use std::{env, fs, path::Path};

use const_gen::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use toml;

// These structs are for determining the version of steckrs.
// The code is based on https://stackoverflow.com/a/75276470
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum DependencyValue
{
	String(String),
	Object
	{
		version: String,
		features: Vec<String>,
	},
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace
{
	dependencies: HashMap<String, DependencyValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CargoToml
{
	workspace: Workspace,
}

pub fn main()
{
	let cargo_toml_raw = include_str!("../Cargo.toml");
	let cargo_toml: CargoToml = toml::from_str(cargo_toml_raw).unwrap();
	let steckrs_dep: &DependencyValue = cargo_toml.workspace.dependencies.get("steckrs").unwrap();
	let steckrs_version_string: String = match steckrs_dep
	{
		DependencyValue::String(val) => val.to_owned(),
		DependencyValue::Object { version, .. } => version.clone(),
	};

	let version_regex: Regex = Regex::new(r"^(\d+)\.(\d+)\.(\d+)$").unwrap();
	let captures = version_regex.captures(&steckrs_version_string).expect(
		format!("steckrs version \"{steckrs_version_string}\" was not in expected format").as_ref(),
	);

	assert!(
		captures.len() == 4,
		"Expected major, minor and patch version numbers from steckrs, but got {} captures",
		captures.len()
	);

	// Compute a version number with 1000 different options per slot
	let major_version: u64 = captures[1].parse::<u64>().unwrap();
	assert!(major_version < 999);

	let minor_version: u64 = captures[2].parse::<u64>().unwrap();
	assert!(minor_version < 999);

	let patch_version: u64 = captures[3].parse::<u64>().unwrap();
	assert!(patch_version < 999);

	let full_version: u64 = (major_version * 1000 * 1000) + (minor_version * 1000) + patch_version;

	// Now we switch over to use const_gen in order to export the version as a const
	// variable. env!() would let us read it as a const string, but we want an
	// integer.
	let out_dir = env::var_os("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("const_gen.rs");
	let const_declarations = vec![const_declaration!(pub FFI_VERSION = full_version)].join("\n");
	fs::write(&dest_path, const_declarations).unwrap();
}
