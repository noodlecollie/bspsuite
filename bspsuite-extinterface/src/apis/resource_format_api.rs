use crate::ApiInfo;
use bspffi::types::{XCBytes, XCSlice, XCStr};
use thin_trait_object::thin_trait_object;

pub const API_INFO: ApiInfo = ApiInfo::new("ResourceFormatApi", 1);
pub type RegisterResourceFormatsFn = extern "C" fn(&mut BoxedResourceFormatApi);
pub type LoadImageFn = extern "C" fn(&XCStr, &mut BoxedImageConstructorApi);

#[repr(C)]
pub enum ImagePixelFormat
{
	R8G8B8,
}

#[repr(C)]
pub struct ResourceFormatApiCallbacks
{
	pub register_resource_formats: RegisterResourceFormatsFn,
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C")]
pub trait ResourceFormatApi
{
	fn register_image_format(
		&mut self,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		load_fn: LoadImageFn,
	);
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C")]
pub trait ImageConstructorApi
{
	fn submit(&mut self, width: u32, height: u32, pixel_format: ImagePixelFormat, data: &XCBytes);
}

impl ImagePixelFormat
{
	pub fn byte_depth(&self) -> u8
	{
		return match self
		{
			ImagePixelFormat::R8G8B8 => 3,
		};
	}
}
