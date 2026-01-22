use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct CompileTuningParametersConfig
{
	pub on_plane_epsilon: Option<f64>,
}

impl CompileTuningParametersConfig
{
	pub fn load(path: &Path) -> Result<Self>
	{
		todo!();
	}

	pub fn merge(existing_cfg: Self, override_cfg: Self) -> Self
	{
		return Self {
			on_plane_epsilon: decide_property(
				existing_cfg.on_plane_epsilon,
				override_cfg.on_plane_epsilon,
			),
		};
	}
}

fn decide_property<T>(existing_prop: Option<T>, override_prop: Option<T>) -> Option<T>
{
	return match override_prop
	{
		Some(prop) => Some(prop),
		None => existing_prop,
	};
}
