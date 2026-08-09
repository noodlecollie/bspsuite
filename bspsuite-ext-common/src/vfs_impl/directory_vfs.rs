use std::debug_assert_eq;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bspextifc::vfs_api::{
	VfsFileErrorCode, VfsFileRecipient, VfsFileRecipientProvider, VfsFileStats, VfsInitResultCode,
	VfsStatRecipient, VfsStatRecipientProvider,
};
use bspffi::types::{XCBytes, XCStr};
use lazy_static::lazy_static;
use log::{error, warn};
use vfs::error::VfsErrorKind;
use vfs::impls::physical::PhysicalFS;
use vfs::{VfsError, VfsFileType, VfsPath};

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
	canonical_root_path: PathBuf,
	vfs: VfsPath,
}

impl VfsRoot
{
	// Assumes root is canonicalised and absolute.
	pub fn new(root: PathBuf) -> Result<Self>
	{
		assert!(root.is_absolute(), "VFS root path must be absolute");

		return Ok(VfsRoot {
			canonical_root_path: root.canonicalize()?,
			vfs: VfsPath::new(PhysicalFS::new(root)),
		});
	}

	pub fn overlaps(&self, other_root: &Path) -> bool
	{
		assert!(
			other_root.is_absolute(),
			"Expected root to be absolute for this comparison to work"
		);

		debug_assert_eq!(
			other_root.canonicalize().unwrap(),
			other_root,
			"Expected root to be canonical for this comparison to work"
		);

		return other_root.starts_with(&self.canonical_root_path)
			|| self.canonical_root_path.starts_with(other_root);
	}

	pub fn exists(&self, sub_path: &str) -> bool
	{
		return self
			.vfs
			.join(sub_path)
			.and_then(|path| path.exists())
			.unwrap_or_else(|err| {
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
			Ok(md) => md.file_type == VfsFileType::File,
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
		};
	}

	pub fn is_directory(&self, sub_path: &str) -> bool
	{
		return match self.vfs.join(sub_path).and_then(|path| path.metadata())
		{
			Ok(md) => md.file_type == VfsFileType::Directory,
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
		};
	}

	pub fn stat(&self, sub_path: &str) -> Result<FileStats, VfsFileErrorCode>
	{
		return self
			.vfs
			.join(sub_path)
			.and_then(|path| Ok((path.clone(), path.metadata()?)))
			.map(|(full_path, md)| {
				let parent_path: VfsPath = full_path.parent();
				let parent_path_string: String = if full_path.is_root() || parent_path.is_root()
				{
					"".to_owned()
				}
				else
				{
					// Trim off any root separator
					parent_path.as_str().trim_start_matches('/').into()
				};
				let file_name: String = full_path.filename();

				FileStats {
					parent_path: parent_path_string,
					name: file_name,
					is_directory: match md.file_type
					{
						VfsFileType::Directory => true,
						_ => false,
					},
					file_size: md.len as usize,
				}
			})
			.map_err(|err| VfsRoot::transform_vfs_error("stat()", err));
	}

	pub fn load_file(&self, sub_path: &str) -> Result<Vec<u8>, VfsFileErrorCode>
	{
		let mut file = self
			.vfs
			.join(sub_path)
			.and_then(|path| path.open_file())
			.map_err(|err| VfsRoot::transform_vfs_error("load_file()", err))?;

		let mut contents: Vec<u8> = Vec::new();

		file.read_to_end(&mut contents).map_err(|err| {
			warn!("Failed to load file {sub_path}: {err}");
			VfsFileErrorCode::IoError
		})?;

		return Ok(contents);
	}

	fn transform_vfs_error(context: &str, err: VfsError) -> VfsFileErrorCode
	{
		return match err.kind()
		{
			VfsErrorKind::FileNotFound | VfsErrorKind::InvalidPath => VfsFileErrorCode::InvalidPath,
			VfsErrorKind::IoError(_) =>
			{
				warn!("{context} encountered unexpected VFS error: {err}");
				VfsFileErrorCode::IoError
			}
			_ =>
			{
				warn!("{context} encountered unexpected VFS error: {err}");
				VfsFileErrorCode::InternalError
			}
		};
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
		if !root.is_dir() || !root.is_absolute()
		{
			warn!(
				"VFS root {} was not an absolute path to a directory",
				root.display()
			);

			return Err(VfsInitResultCode::InvalidRootPath);
		}

		let canonical_root: PathBuf = root.canonicalize().map_err(|err| {
			warn!("Failed to canonicalise VFS root {}: {err}", root.display());
			VfsInitResultCode::InvalidRootPath
		})?;

		for existing in self.vfs.iter()
		{
			if existing.canonical_root_path == canonical_root
			{
				return Err(VfsInitResultCode::RootAlreadyInUse);
			}

			if existing.overlaps(canonical_root.as_path())
			{
				return Err(VfsInitResultCode::RootsOverlap);
			}
		}

		let vfs_root: VfsRoot = VfsRoot::new(root.clone()).map_err(|err| {
			error!("Failed to create VFS root under {}: {err}", root.display());
			VfsInitResultCode::InternalError
		})?;

		self.vfs.push(vfs_root);
		return Ok(());
	}

	pub fn exists(&self, sub_path: &str) -> bool
	{
		return self.vfs.iter().rev().any(|vfs| vfs.exists(sub_path));
	}

	pub fn is_file(&self, sub_path: &str) -> bool
	{
		return self.vfs.iter().rev().any(|vfs| vfs.is_file(sub_path));
	}

	pub fn is_directory(&self, sub_path: &str) -> bool
	{
		return self.vfs.iter().rev().any(|vfs| vfs.is_directory(sub_path));
	}

	pub fn stat(&self, sub_path: &str) -> Result<FileStats, VfsFileErrorCode>
	{
		for vfs in self.vfs.iter().rev()
		{
			match vfs.stat(sub_path)
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

	pub fn load_file(&self, sub_path: &str) -> Result<Vec<u8>, VfsFileErrorCode>
	{
		for vfs in self.vfs.iter().rev()
		{
			match vfs.load_file(sub_path)
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
		|vfs| vfs.exists(path.as_str()),
	);
}

pub(super) extern "C" fn is_file(path: &XCStr) -> bool
{
	return static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			false
		},
		|vfs| vfs.is_file(path.as_str()),
	);
}

pub(super) extern "C" fn is_directory(path: &XCStr) -> bool
{
	return static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			false
		},
		|vfs| vfs.is_directory(path.as_str()),
	);
}

pub(super) extern "C" fn stat(path: &XCStr, recipient: &mut VfsStatRecipientProvider)
{
	let result: Result<FileStats, VfsFileErrorCode> = static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			Err(VfsFileErrorCode::InternalError)
		},
		|vfs| vfs.stat(path.as_str()),
	);

	match result
	{
		Ok(stats) => recipient.submit_stats(&VfsFileStats::from(&stats)),
		Err(err) => recipient.set_error(err),
	};
}

pub(super) extern "C" fn load_file(path: &XCStr, recipient: &mut VfsFileRecipientProvider)
{
	let result: Result<Vec<u8>, VfsFileErrorCode> = static_vfs.lock().map_or_else(
		|err| {
			error!("Unable to acquire VFS mutex: {err}");
			Err(VfsFileErrorCode::InternalError)
		},
		|vfs| vfs.load_file(path.as_str()),
	);

	match result
	{
		Ok(bytes) => recipient.submit_bytes(&XCBytes::from(bytes.as_slice())),
		Err(err) => recipient.set_error(err),
	};
}

#[cfg(test)]
mod tests
{
	use std::assert_eq;

	use super::*;
	use std::fs;
	use target_test_dir::with_test_dir;

	#[test]
	#[with_test_dir]
	fn overlaps()
	{
		let testdir = get_test_dir!();

		fs::create_dir_all(testdir.join("root1/dir/subdir/")).unwrap();
		fs::create_dir_all(testdir.join("root2/dir/subdir")).unwrap();

		let canonical_root: PathBuf = testdir.join("root1/dir/subdir/").canonicalize().unwrap();
		let root: VfsRoot = VfsRoot::new(canonical_root.clone()).unwrap();

		let canon = |path: &str| testdir.join(path).canonicalize().unwrap();

		assert!(root.overlaps(canon("root1/dir/subdir").as_path()));
		assert!(root.overlaps(canon("root1/dir/").as_path()));
		assert!(root.overlaps(canon("root1/").as_path()));
		assert!(root.overlaps(canonical_root.as_path()));

		assert!(!root.overlaps(canon("root2/dir/subdir").as_path()));
		assert!(!root.overlaps(canon("root2/dir/").as_path()));
		assert!(!root.overlaps(canon("root2/").as_path()));
	}

	#[test]
	#[with_test_dir]
	fn independent_filesystem_roots()
	{
		let testdir = get_test_dir!();
		create_test_filesystem(testdir.as_path());

		let root1: VfsRoot = VfsRoot::new(testdir.join("root1")).unwrap();
		let root2: VfsRoot = VfsRoot::new(testdir.join("root2")).unwrap();

		// All valid file paths should exist
		assert!(root1.exists(""));
		assert!(root1.exists("file1"));
		assert!(root1.exists("file2"));
		assert!(root1.exists("subdir"));
		assert!(root1.exists("subdir/file3"));

		assert!(root1.is_file("file1"));
		assert!(root1.is_file("file2"));
		assert!(root1.is_file("subdir/file3"));
		assert!(root1.is_directory(""));
		assert!(root1.is_directory("subdir"));

		assert!(root2.exists(""));
		assert!(root2.exists("file4"));
		assert!(root2.exists("file5"));
		assert!(root2.exists("subdir"));
		assert!(root2.exists("subdir/file6"));

		assert!(root2.is_file("file4"));
		assert!(root2.is_file("file5"));
		assert!(root2.is_file("subdir/file6"));
		assert!(root2.is_directory(""));
		assert!(root2.is_directory("subdir"));

		// Files should not cross-pollinate
		assert!(!root2.exists("file1"));
		assert!(!root2.exists("file2"));
		assert!(!root2.exists("subdir/file3"));
		assert!(!root1.exists("file4"));
		assert!(!root1.exists("file5"));
		assert!(!root1.exists("subdir/file6"));

		// Stats should be reported as expected
		assert_eq!(
			root1.stat("file1").unwrap(),
			FileStats {
				parent_path: "".to_owned(),
				name: "file1".to_owned(),
				is_directory: false,
				file_size: 14
			}
		);

		assert_eq!(
			root1.stat("file2").unwrap(),
			FileStats {
				parent_path: "".to_owned(),
				name: "file2".to_owned(),
				is_directory: false,
				file_size: 14
			}
		);

		assert_eq!(
			root1.stat("subdir/file3").unwrap(),
			FileStats {
				parent_path: "subdir".to_owned(),
				name: "file3".to_owned(),
				is_directory: false,
				file_size: 36
			}
		);

		assert_eq!(
			root1.stat("").unwrap(),
			FileStats {
				parent_path: "".to_owned(),
				name: "".to_owned(),
				is_directory: true,
				file_size: 0
			}
		);

		assert_eq!(
			root1.stat("subdir").unwrap(),
			FileStats {
				parent_path: "".to_owned(),
				name: "subdir".to_owned(),
				is_directory: true,
				file_size: 0
			}
		);

		// Backing up outside the root should not be allowed.
		assert!(fs::exists(testdir.join("outside_roots")).unwrap());

		assert!(!root1.exists("../outside_roots"));
		assert!(!root1.is_file("../outside_roots"));
		assert!(!root1.is_directory("../outside_roots"));
		assert!(matches!(
			root1.stat("../outside_roots").unwrap_err(),
			VfsFileErrorCode::InvalidPath
		));
		assert!(matches!(
			root1.load_file("../outside_roots").unwrap_err(),
			VfsFileErrorCode::InvalidPath
		));

		assert!(!root2.exists("../outside_roots"));
		assert!(!root2.is_file("../outside_roots"));
		assert!(!root2.is_directory("../outside_roots"));
		assert!(matches!(
			root2.stat("../outside_roots").unwrap_err(),
			VfsFileErrorCode::InvalidPath
		));
		assert!(matches!(
			root2.load_file("../outside_roots").unwrap_err(),
			VfsFileErrorCode::InvalidPath
		));
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
		assert!(vfs.exists("file1"));
		assert!(vfs.exists("file2"));
		assert!(vfs.exists("subdir/file3"));

		assert!(vfs.is_file("file1"));
		assert!(vfs.is_file("file2"));
		assert!(vfs.is_file("subdir/file3"));

		assert!(vfs.exists("file4"));
		assert!(vfs.exists("file5"));
		assert!(vfs.exists("subdir/file6"));

		assert!(vfs.is_file("file4"));
		assert!(vfs.is_file("file5"));
		assert!(vfs.is_file("subdir/file6"));

		// The subdirectory is present under both roots - this is fine.
		assert!(vfs.is_directory("subdir"));

		// The common file should exist.
		assert!(vfs.is_file("common_file"));

		let contents: Vec<u8> = vfs.load_file("common_file").unwrap();

		// We expect the common file to be loaded from root 2, as this was added later
		// and therefore overlays on top of root 1.
		assert_eq!(
			contents,
			"This is a file with a common name, but in root 2".as_bytes()
		);
	}

	fn create_test_filesystem(testdir: &Path)
	{
		fs::write(
			testdir.join("outside_roots"),
			"This is a file outside of both roots",
		)
		.unwrap();

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
