use std::ffi::OsStr;
use std::fs;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bspextifc::vfs_api::{
	BoxedVfsFileRecipient, BoxedVfsStatRecipient, VfsFileErrorCode, VfsFileRecipient,
	VfsFileRecipientResult, VfsFileStats, VfsInitResultCode, VfsStatRecipient,
};
use bspffi::types::{XCBytes, XCStr};
use filesize::PathExt;
use lazy_static::lazy_static;
use log::{error, warn};

lazy_static! {
	static ref static_vfs: Mutex<VfsRootContainer> = Mutex::new(VfsRootContainer::new());
}

struct VfsRoot
{
	root: PathBuf,
}

impl VfsRoot
{
	pub fn new(root: PathBuf) -> Self
	{
		return Self { root };
	}

	pub fn overlaps(&self, other_root: &Path) -> bool
	{
		return other_root.starts_with(&self.root) || self.root.starts_with(other_root);
	}

	pub fn exists(&self, sub_path: &Path) -> bool
	{
		if sub_path.has_root()
		{
			return false;
		}

		return self.root.join(sub_path).exists();
	}

	pub fn is_file(&self, sub_path: &Path) -> bool
	{
		if sub_path.has_root()
		{
			return false;
		}

		return self.root.join(sub_path).is_file();
	}

	pub fn is_directory(&self, sub_path: &Path) -> bool
	{
		if sub_path.has_root()
		{
			return false;
		}

		return self.root.join(sub_path).is_dir();
	}

	pub fn stat(
		&self,
		sub_path: &Path,
		recipient: &mut BoxedVfsStatRecipient,
	) -> Result<(), VfsFileErrorCode>
	{
		if sub_path.has_root() || !self.exists(sub_path)
		{
			return Err(VfsFileErrorCode::InvalidPath);
		}

		let name: &OsStr = sub_path
			.file_name()
			.ok_or_else(|| VfsFileErrorCode::InvalidPath)?;

		let full_path: PathBuf = self.root.join(sub_path);

		let metadata = full_path
			.symlink_metadata()
			.map_err(|_| VfsFileErrorCode::InternalError)?;

		let real_size: u64 = full_path
			.size_on_disk_fast(&metadata)
			.map_err(|_| VfsFileErrorCode::InternalError)?;

		let parent_path: PathBuf = sub_path
			.parent()
			.map(|path| PathBuf::from(path))
			.unwrap_or(PathBuf::from(""));

		let stats: VfsFileStats = VfsFileStats {
			parent_path: XCStr::from(
				parent_path
					.to_str()
					.ok_or_else(|| VfsFileErrorCode::InternalError)?,
			),
			name: XCStr::from(
				name.to_str()
					.ok_or_else(|| VfsFileErrorCode::InternalError)?,
			),
			is_directory: full_path.is_dir(),
			file_size: real_size as usize,
		};

		recipient.submit_stats(&stats);
		return Ok(());
	}

	pub fn load_file(&self, sub_path: &Path) -> Result<Vec<u8>, VfsFileErrorCode>
	{
		if sub_path.has_root() || !self.exists(sub_path)
		{
			return Err(VfsFileErrorCode::InvalidPath);
		}

		let full_path: PathBuf = self.root.join(sub_path);

		return fs::read(full_path.as_path()).map_err(|err| {
			warn!("Failed to read {}: {err}", full_path.display());
			VfsFileErrorCode::IoError
		});
	}
}

struct VfsRootContainer
{
	vfs: Vec<VfsRoot>,
}

impl VfsRootContainer
{
	pub fn new() -> Self
	{
		return Self { vfs: Vec::new() };
	}

	pub fn add(&mut self, root: PathBuf) -> Result<(), VfsInitResultCode>
	{
		if !root.is_dir()
		{
			return Err(VfsInitResultCode::InvalidRootPath);
		}

		for existing in self.vfs.iter()
		{
			if existing.root == root
			{
				return Err(VfsInitResultCode::RootAlreadyInUse);
			}

			if existing.overlaps(root.as_path())
			{
				return Err(VfsInitResultCode::RootsOverlap);
			}
		}

		self.vfs.push(VfsRoot::new(root));
		return Ok(());
	}

	pub fn exists(&self, path: &Path) -> bool
	{
		return self.vfs.iter().any(|vfs| vfs.exists(path));
	}

	pub fn is_file(&self, path: &Path) -> bool
	{
		return self.vfs.iter().any(|vfs| vfs.is_file(path));
	}

	pub fn is_directory(&self, path: &Path) -> bool
	{
		return self.vfs.iter().any(|vfs| vfs.is_directory(path));
	}

	pub fn stat(
		&self,
		path: &Path,
		recipient: &mut BoxedVfsStatRecipient,
	) -> Result<(), VfsFileErrorCode>
	{
		for vfs in self.vfs.iter()
		{
			match vfs.stat(path, recipient)
			{
				Err(err) =>
				{
					if let VfsFileErrorCode::InvalidPath = err
					{
						// Root did not recognise this path, so try the next one.
						continue;
					}

					return Err(err);
				}
				Ok(_) => return Ok(()),
			};
		}

		return Err(VfsFileErrorCode::InvalidPath);
	}

	pub fn load_file(&self, path: &Path) -> Result<Vec<u8>, VfsFileErrorCode>
	{
		for vfs in self.vfs.iter()
		{
			match vfs.load_file(path)
			{
				Err(err) =>
				{
					if let VfsFileErrorCode::InvalidPath = err
					{
						// Root did not recognise this path, so try the next one.
						continue;
					}

					return Err(err);
				}
				Ok(val) => return Ok(val),
			};
		}

		return Err(VfsFileErrorCode::InvalidPath);
	}
}

pub(super) extern "C" fn initialise(real_root_node: &XCStr) -> VfsInitResultCode
{
	return static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			VfsInitResultCode::InternalError
		},
		|mut vfs| {
			vfs.deref_mut()
				.add(PathBuf::from(real_root_node.as_str()))
				.err()
				.unwrap_or(VfsInitResultCode::Ok)
		},
	);
}

pub(super) extern "C" fn exists(path: &XCStr) -> bool
{
	return static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			false
		},
		|vfs| vfs.exists(PathBuf::from(path.as_str()).as_path()),
	);
}

pub(super) extern "C" fn is_file(path: &XCStr) -> bool
{
	return static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			false
		},
		|vfs| vfs.is_file(PathBuf::from(path.as_str()).as_path()),
	);
}

pub(super) extern "C" fn is_directory(path: &XCStr) -> bool
{
	return static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			false
		},
		|vfs| vfs.is_directory(PathBuf::from(path.as_str()).as_path()),
	);
}

pub(super) extern "C" fn stat(path: &XCStr, recipient: &mut BoxedVfsStatRecipient)
{
	let result: Result<(), VfsFileErrorCode> = static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			Err(VfsFileErrorCode::InternalError)
		},
		|vfs| vfs.stat(PathBuf::from(path.as_str()).as_path(), recipient),
	);

	if let Err(err) = result
	{
		recipient.set_error(err);
	}
}

pub(super) extern "C" fn load_file(path: &XCStr, recipient: &mut BoxedVfsFileRecipient)
{
	let result: Result<Vec<u8>, VfsFileErrorCode> = static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			Err(VfsFileErrorCode::InternalError)
		},
		|vfs| vfs.load_file(PathBuf::from(path.as_str()).as_path()),
	);

	match result
	{
		Ok(bytes) =>
		{
			if let VfsFileRecipientResult::AlreadyHasResult =
				recipient.submit_bytes(&XCBytes::from(bytes.as_slice()))
			{
				warn!("Unexpected result already set on file recipient");
			}
		}
		Err(err) => recipient.set_error(err),
	};
}

#[cfg(test)]
mod tests
{
	use super::*;
	use target_test_dir::with_test_dir;

	#[test]
	fn overlaps()
	{
		let root: VfsRoot = VfsRoot::new(PathBuf::from("/path/to/my/dir"));

		assert!(root.overlaps(PathBuf::from("/path/to/my/dir").as_path()));
		assert!(root.overlaps(PathBuf::from("/path/to/my").as_path()));
		assert!(root.overlaps(PathBuf::from("/path/to").as_path()));
		assert!(root.overlaps(PathBuf::from("/path").as_path()));
		assert!(root.overlaps(PathBuf::from("/").as_path()));
		assert!(root.overlaps(PathBuf::from("/path/to/my/dir/subdir").as_path()));

		assert!(!root.overlaps(PathBuf::from("/path/to/somewhere/else").as_path()));
	}

	#[test]
	#[with_test_dir]
	fn independent_filesystem_roots()
	{
		let testdir = get_test_dir!();
		create_test_filesystem(testdir.as_path());

		let root1: VfsRoot = VfsRoot::new(testdir.join("root1"));
		let root2: VfsRoot = VfsRoot::new(testdir.join("root2"));

		// All valid file paths should exist
		assert!(root1.exists(PathBuf::from("file1").as_path()));
		assert!(root1.exists(PathBuf::from("file2").as_path()));
		assert!(root1.exists(PathBuf::from("subdir/file3").as_path()));

		assert!(root1.is_file(PathBuf::from("file1").as_path()));
		assert!(root1.is_file(PathBuf::from("file2").as_path()));
		assert!(root1.is_file(PathBuf::from("subdir/file3").as_path()));
		assert!(root1.is_directory(PathBuf::from("subdir").as_path()));

		assert!(root2.exists(PathBuf::from("file4").as_path()));
		assert!(root2.exists(PathBuf::from("file5").as_path()));
		assert!(root2.exists(PathBuf::from("subdir/file6").as_path()));

		assert!(root2.is_file(PathBuf::from("file4").as_path()));
		assert!(root2.is_file(PathBuf::from("file5").as_path()));
		assert!(root2.is_file(PathBuf::from("subdir/file6").as_path()));
		assert!(root2.is_directory(PathBuf::from("subdir").as_path()));

		// Files should not cross-pollinate
		assert!(!root2.exists(PathBuf::from("file1").as_path()));
		assert!(!root2.exists(PathBuf::from("file2").as_path()));
		assert!(!root2.exists(PathBuf::from("subdir/file3").as_path()));
		assert!(!root1.exists(PathBuf::from("file4").as_path()));
		assert!(!root1.exists(PathBuf::from("file5").as_path()));
		assert!(!root1.exists(PathBuf::from("subdir/file6").as_path()));
	}

	#[test]
	#[with_test_dir]
	fn combined_vfs()
	{
		let testdir = get_test_dir!();
		create_test_filesystem(testdir.as_path());

		let mut vfs: VfsRootContainer = VfsRootContainer::new();
		vfs.add(testdir.join("root1")).unwrap();
		vfs.add(testdir.join("root2")).unwrap();

		// All valid file paths should exist
		assert!(vfs.exists(PathBuf::from("file1").as_path()));
		assert!(vfs.exists(PathBuf::from("file2").as_path()));
		assert!(vfs.exists(PathBuf::from("subdir/file3").as_path()));

		assert!(vfs.is_file(PathBuf::from("file1").as_path()));
		assert!(vfs.is_file(PathBuf::from("file2").as_path()));
		assert!(vfs.is_file(PathBuf::from("subdir/file3").as_path()));

		assert!(vfs.exists(PathBuf::from("file4").as_path()));
		assert!(vfs.exists(PathBuf::from("file5").as_path()));
		assert!(vfs.exists(PathBuf::from("subdir/file6").as_path()));

		assert!(vfs.is_file(PathBuf::from("file4").as_path()));
		assert!(vfs.is_file(PathBuf::from("file5").as_path()));
		assert!(vfs.is_file(PathBuf::from("subdir/file6").as_path()));

		// The subdirectory is present under both roots - this is fine.
		assert!(vfs.is_directory(PathBuf::from("subdir").as_path()));

		// The common file should exist.
		assert!(vfs.is_file(PathBuf::from("common_file").as_path()));

		let contents: Vec<u8> = vfs
			.load_file(PathBuf::from("common_file").as_path())
			.unwrap();

		// We expect the common file to be loaded from root 1, as this was the first
		// root that was added.
		assert_eq!(
			contents,
			"This is a file with a common name, but in root 1".as_bytes()
		);
	}

	fn create_test_filesystem(testdir: &Path)
	{
		fs::create_dir_all(testdir.join("root1")).unwrap();
		fs::create_dir_all(testdir.join("root2")).unwrap();
		fs::write(testdir.join("root1/file1"), "This is file 1").unwrap();
		fs::write(testdir.join("root1/file2"), "This is file 2").unwrap();
		fs::create_dir_all(testdir.join("root1/subdir")).unwrap();
		fs::write(
			testdir.join("root1/subdir/file3"),
			"This is file 3, which is in a subdir",
		)
		.unwrap();
		fs::write(testdir.join("root2/file4"), "This is file 4").unwrap();
		fs::write(testdir.join("root2/file5"), "This is file 5").unwrap();
		fs::create_dir_all(testdir.join("root2/subdir")).unwrap();
		fs::write(
			testdir.join("root2/subdir/file6"),
			"This is file 6, which is in a subdir",
		)
		.unwrap();

		fs::write(
			testdir.join("root1/common_file"),
			"This is a file with a common name, but in root 1",
		)
		.unwrap();

		fs::write(
			testdir.join("root2/common_file"),
			"This is a file with a common name, but in root 2",
		)
		.unwrap();
	}
}
