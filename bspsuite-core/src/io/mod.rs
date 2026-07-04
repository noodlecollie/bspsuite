// TODO: serde implementation using a newtype wrapper for each item?
// Alternatively, versioned reader and writer objects which implement serde?
pub use map_geometry_file::deserialize as deserialize_map_geometry_file;
pub use map_geometry_file::read as read_map_geometry_file;
pub use map_geometry_file::serialize as serialize_map_geometry_file;
pub use map_geometry_file::write as write_map_geometry_file;

pub use map_source_file::deserialize as deserialize_map_source_file;
pub use map_source_file::read as read_map_source_file;
pub use map_source_file::serialize as serialize_map_source_file;
pub use map_source_file::write as write_map_source_file;

mod json_utils;
mod map_geometry_file;
mod map_source_file;
