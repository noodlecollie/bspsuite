use std::ffi::OsStr;
use std::fs;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Result, bail, ensure};
use bspextifc::vfs_api::{
	BoxedVfsFileRecipient, BoxedVfsStatRecipient, VfsFileErrorCode, VfsFileRecipient,
	VfsFileRecipientResult, VfsFileStats, VfsInitResultCode, VfsStatRecipient,
};
use bspffi::types::{XCBytes, XCStr};
use filesize::PathExt;
use lazy_static::lazy_static;
use log::{error, warn};
use vfs::error::VfsErrorKind;
use vfs::{FileSystem, VfsError, VfsFileType, VfsMetadata, VfsPath};
use vfs::impls::altroot::AltrootFS;
use vfs::impls::physical::PhysicalFS;

lazy_static! {
	static ref static_vfs: Mutex<VfsRootContainer> = Mutex::new(VfsRootContainer::new());
}

#[derive(Debug, PartialEq)]
struct FileStats
{
	pub parent_path: String,
	pub name: String,
	pub is_directory: bool,
	pub file_size: usize,
}

impl<'l> From<&'l FileStats> for VfsFileStats<'l>
{
	fn from(value: &'l FileStats) -> Self
	{
		return Self {
			parent_path: XCStr::from(value.parent_path.as_str()),
			name: XCStr::from(value.name.as_str()),
			is_directory: value.is_directory,
			file_size: value.file_size,
		};
	}
}

struct VfsRoot
{
	root_path: PathBuf,
	vfs: VfsPath,
}

impl VfsRoot
{
	pub fn new(root: PathBuf) -> Result<Self>
	{
		ensure!(root.is_absolute(), "VFS root path must be absolute");

		let canonical_root: PathBuf = root.canonicalize()?;
		let physical_fs: PhysicalFS = PhysicalFS::new(canonical_root.clone());

		return Ok(VfsRoot { root_path: canonical_root, vfs: VfsPath::new(physical_fs) });
	}

	pub fn overlaps(&self, other_root: &Path) -> bool
	{
		assert!(other_root.is_absolute(), "Expected root to be absolute for this comparison to work");
		return other_root.starts_with(&self.root_path) || self.root_path.starts_with(other_root);
	}

	pub fn exists(&self, sub_path: &str) -> bool
	{
		return self.vfs.join(sub_path).and_then(|path| path.exists()).unwrap_or_else(|err|
			{
				match err.kind()
				{
					VfsErrorKind::FileNotFound | VfsErrorKind::InvalidPath => (),
					_ =>
					{
						warn!("Unexpected VFS error: {err}");
					}
				};

				false
			});
	}

	pub fn is_file(&self, sub_path: &str) -> bool
	{
		return match self.vfs.join(sub_path).and_then(|path| path.metadata())
		{
			Ok(md) =>
			{
				md.file_type == VfsFileType::File
			},
			Err(err) =>
			{
				match err.kind()
				{
					VfsErrorKind::FileNotFound | VfsErrorKind::InvalidPath => (),
					_ =>
					{
						warn!("Unexpected VFS error: {err}");
					}
				};

				false
			}
		}
	}

	pub fn is_directory(&self, sub_path: &str) -> bool
	{
		return match self.vfs.join(sub_path).and_then(|path| path.metadata())
		{
			Ok(md) =>
			{
				md.file_type == VfsFileType::Directory
			},
			Err(err) =>
			{
				match err.kind()
				{
					VfsErrorKind::FileNotFound | VfsErrorKind::InvalidPath => (),
					_ =>
					{
						warn!("Unexpected VFS error: {err}");
					}
				};

				false
			}
		}
	}

	pub fn stat(&self, sub_path: &str) -> Result<FileStats, VfsFileErrorCode>
	{
		return self.vfs.join(sub_path).and_then(|path| Ok((path.clone(), path.metadata()?))).map(|(full_path, md)|
			{
				let parent_path: VfsPath = full_path.parent();
				let parent_path_string: String = if full_path.is_root() { String::new() } else {parent_path.as_str().into()};
				let file_name: String = full_path.filename();

				FileStats{
					parent_path: parent_path_string,
					name: file_name,
					is_directory: match md.file_type
					{
						VfsFileType::Directory => true,
						_ => false
					},
					file_size: md.len as usize
				}
			}).map_err(|err|
		{
			match err.kind()
			{
				VfsErrorKind::FileNotFound | VfsErrorKind::InvalidPath => VfsFileErrorCode::InvalidPath,
				VfsErrorKind::IoError(_) =>
				{
					warn!("Unexpected VFS error: {err}");
					VfsFileErrorCode::IoError
				},
				_ =>
				{
					warn!("Unexpected VFS error: {err}");
					VfsFileErrorCode::InternalError
				}
			}
		});
	}

	pub fn load_file(&self, _sub_path: &Path) -> Result<Vec<u8>, VfsFileErrorCode>
	{
		// TODO
		return Err(VfsFileErrorCode::InternalError);
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
			if existing.root_path == root
			{
				return Err(VfsInitResultCode::RootAlreadyInUse);
			}

			if existing.overlaps(root.as_path())
			{
				return Err(VfsInitResultCode::RootsOverlap);
			}
		}

		// TODO
		// self.vfs.push(VfsRoot::new(root));
		// return Ok(());
		return Err(VfsInitResultCode::InternalError);
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

	pub fn stat(&self, path: &Path) -> Result<FileStats, VfsFileErrorCode>
	{
		for vfs in self.vfs.iter()
		{
			match vfs.stat(path)
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
				Ok(stats) => return Ok(stats),
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
	let result: Result<FileStats, VfsFileErrorCode> = static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			Err(VfsFileErrorCode::InternalError)
		},
		|vfs| vfs.stat(PathBuf::from(path.as_str()).as_path()),
	);

	match result
	{
		Ok(stats) =>
		{
			if let VfsFileRecipientResult::AlreadyHasResult =
				recipient.submit_stats(&VfsFileStats::from(&stats))
			{
				warn!("Unexpected result already set on stats recipient");
			}
		}
		Err(err) => recipient.set_error(err),
	};
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

	// #[test]
	// fn overlaps()
	// {
	// 	let root: VfsRoot = VfsRoot::new(PathBuf::from("/path/to/my/dir"));

	// 	assert!(root.overlaps(PathBuf::from("/path/to/my/dir").as_path()));
	// 	assert!(root.overlaps(PathBuf::from("/path/to/my").as_path()));
	// 	assert!(root.overlaps(PathBuf::from("/path/to").as_path()));
	// 	assert!(root.overlaps(PathBuf::from("/path").as_path()));
	// 	assert!(root.overlaps(PathBuf::from("/").as_path()));
	// 	assert!(root.overlaps(PathBuf::from("/path/to/my/dir/subdir").as_path()));

	// 	assert!(!root.overlaps(PathBuf::from("/path/to/somewhere/else").as_path()));
	// }

	// TODO: Testing paths that go up outside the root

	// #[test]
	// #[with_test_dir]
	// fn independent_filesystem_roots()
	// {
	// 	let testdir = get_test_dir!();
	// 	create_test_filesystem(testdir.as_path());

	// 	let root1: VfsRoot = VfsRoot::new(testdir.join("root1"));
	// 	let root2: VfsRoot = VfsRoot::new(testdir.join("root2"));

	// 	// All valid file paths should exist
	// 	assert!(root1.exists(PathBuf::from("").as_path()));
	// 	assert!(root1.exists(PathBuf::from("file1").as_path()));
	// 	assert!(root1.exists(PathBuf::from("file2").as_path()));
	// 	assert!(root1.exists(PathBuf::from("subdir").as_path()));
	// 	assert!(root1.exists(PathBuf::from("subdir/file3").as_path()));

	// 	assert!(root1.is_file(PathBuf::from("file1").as_path()));
	// 	assert!(root1.is_file(PathBuf::from("file2").as_path()));
	// 	assert!(root1.is_file(PathBuf::from("subdir/file3").as_path()));
	// 	assert!(root1.is_directory(PathBuf::from("").as_path()));
	// 	assert!(root1.is_directory(PathBuf::from("subdir").as_path()));

	// 	assert!(root2.exists(PathBuf::from("").as_path()));
	// 	assert!(root2.exists(PathBuf::from("file4").as_path()));
	// 	assert!(root2.exists(PathBuf::from("file5").as_path()));
	// 	assert!(root2.exists(PathBuf::from("subdir").as_path()));
	// 	assert!(root2.exists(PathBuf::from("subdir/file6").as_path()));

	// 	assert!(root2.is_file(PathBuf::from("file4").as_path()));
	// 	assert!(root2.is_file(PathBuf::from("file5").as_path()));
	// 	assert!(root2.is_file(PathBuf::from("subdir/file6").as_path()));
	// 	assert!(root2.is_directory(PathBuf::from("").as_path()));
	// 	assert!(root2.is_directory(PathBuf::from("subdir").as_path()));

	// 	// Files should not cross-pollinate
	// 	assert!(!root2.exists(PathBuf::from("file1").as_path()));
	// 	assert!(!root2.exists(PathBuf::from("file2").as_path()));
	// 	assert!(!root2.exists(PathBuf::from("subdir/file3").as_path()));
	// 	assert!(!root1.exists(PathBuf::from("file4").as_path()));
	// 	assert!(!root1.exists(PathBuf::from("file5").as_path()));
	// 	assert!(!root1.exists(PathBuf::from("subdir/file6").as_path()));

	// 	// Stats should be reported as expected
	// 	assert_eq!(
	// 		root1.stat(PathBuf::from("file1").as_path()).unwrap(),
	// 		FileStats {
	// 			parent_path: "".to_owned(),
	// 			name: "file1".to_owned(),
	// 			is_directory: false,
	// 			file_size: 14
	// 		}
	// 	);

	// 	assert_eq!(
	// 		root1.stat(PathBuf::from("file2").as_path()).unwrap(),
	// 		FileStats {
	// 			parent_path: "".to_owned(),
	// 			name: "file2".to_owned(),
	// 			is_directory: false,
	// 			file_size: 14
	// 		}
	// 	);

	// 	assert_eq!(
	// 		root1.stat(PathBuf::from("subdir/file3").as_path()).unwrap(),
	// 		FileStats {
	// 			parent_path: "subdir".to_owned(),
	// 			name: "file3".to_owned(),
	// 			is_directory: false,
	// 			file_size: 36
	// 		}
	// 	);

	// 	assert_eq!(
	// 		root1.stat(PathBuf::from("").as_path()).unwrap(),
	// 		FileStats {
	// 			parent_path: "".to_owned(),
	// 			name: "".to_owned(),
	// 			is_directory: true,
	// 			file_size: 0
	// 		}
	// 	);

	// 	assert_eq!(
	// 		root1.stat(PathBuf::from("subdir").as_path()).unwrap(),
	// 		FileStats {
	// 			parent_path: "".to_owned(),
	// 			name: "subdir".to_owned(),
	// 			is_directory: true,
	// 			file_size: 0
	// 		}
	// 	);
	// }

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
