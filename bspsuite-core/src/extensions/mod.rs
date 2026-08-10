pub(crate) use extension::{Extension, ExtensionCollection, ExtensionData};
pub(crate) use format_loader::{
	ApiCollector, FormatCollector, FormatLoader, FormatLoaderEndpoint, FormatSpec,
};

use file_format_collection::ExtensionFileFormatCollection;

mod api_impl;
mod extension;
mod file_format_collection;
mod format_loader;

pub(crate) use api_impl::VfsFileStatResult;

#[cfg(test)]
mod api_tests;
