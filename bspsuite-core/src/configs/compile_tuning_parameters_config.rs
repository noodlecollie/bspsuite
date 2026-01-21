use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CompileTuningParametersConfig
{
	pub on_plane_epsilon: Option<f64>,
}
