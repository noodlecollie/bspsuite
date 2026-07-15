use bspextifc::vfs_api::{
	BoxedVfsFileRecipient, BoxedVfsStatRecipient, VfsFileErrorCode, VfsFileRecipient,
	VfsInitResultCode, VfsStatRecipient,
};
use bspffi::types::XCStr;

pub(super) extern "C" fn initialise(_real_root_node: &XCStr) -> VfsInitResultCode
{
	// TODO
	return VfsInitResultCode::InternalError;
}

pub(super) extern "C" fn exists(_path: &XCStr) -> bool
{
	// TODO
	return false;
}

pub(super) extern "C" fn is_file(_path: &XCStr) -> bool
{
	// TODO
	return false;
}

pub(super) extern "C" fn is_directory(_path: &XCStr) -> bool
{
	// TODO
	return false;
}

pub(super) extern "C" fn stat(_path: &XCStr, recipient: &mut BoxedVfsStatRecipient)
{
	// TODO
	recipient.set_error(VfsFileErrorCode::InternalError);
}

pub(super) extern "C" fn load_file(_path: &XCStr, recipient: &mut BoxedVfsFileRecipient)
{
	// TODO
	recipient.set_error(VfsFileErrorCode::InternalError);
}
