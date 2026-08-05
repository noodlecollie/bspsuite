pub(crate) use extension::{Extension, ExtensionCollection};
pub(crate) use format_loader::{FormatCollector, FormatLoader, FormatLoaderEndpoint, FormatSpec};

use file_format_collection::ExtensionFileFormatCollection;

mod api_impl;
mod extension;
mod file_format_collection;
mod format_loader;
