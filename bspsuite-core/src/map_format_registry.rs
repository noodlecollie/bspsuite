use crate::extensions::ExtensionResource;
use anyhow::{Result, bail};
use bspextifc::map_format_api::MapParseFn;
use std::collections::HashMap;

pub type MapFormatEntry = ExtensionResource<MapParseFn>;

pub struct MapFormatRegistry
{
	map: HashMap<String, Vec<MapFormatEntry>>,
}

impl MapFormatRegistry
{
	pub fn new() -> Self
	{
		return Self {
			map: HashMap::new(),
		};
	}

	pub fn insert(&mut self, format_name: String, entry: MapFormatEntry) -> Result<()>
	{
		if let Some(entry_list) = self.map.get_mut(&format_name)
		{
			let extension_name: &str = entry.extension().get_name();

			if entry_list
				.iter()
				.find(|existing_entry| existing_entry.extension().get_name() == extension_name)
				.is_none()
			{
				entry_list.push(entry);
			}
			else
			{
				bail!(
					"Ignoring attempt to register map format {format_name} twice for extension {extension_name}"
				);
			}
		}
		else
		{
			let mut entry_list: Vec<MapFormatEntry> = Vec::new();
			entry_list.push(entry);
			self.map.insert(format_name, entry_list);
		}

		return Ok(());
	}

	pub fn get_parsers(&self, format_name: &str) -> Option<&[MapFormatEntry]>
	{
		return self.map.get(format_name).map(|entry| entry.as_slice());
	}
}
