use std::fs;
use std::path::PathBuf;

use super::super::extension::full_shared_library_name as extension_lib_name;
use crate::extensions::{ExtensionCollection, VfsFileStatResult};
use target_test_dir::with_test_dir;

#[test]
#[with_test_dir]
fn general_directory_vfs_tests()
{
	let testdir = get_test_dir!();

	fs::write(testdir.join("file1"), "This is file 1").unwrap();
	fs::write(testdir.join("file2"), "This is file 2").unwrap();
	fs::create_dir_all(testdir.join("subdir")).unwrap();
	fs::write(
		testdir.join("subdir/file3"),
		"This is file 3, which is in a subdir",
	)
	.unwrap();

	let lib_path: PathBuf = testdir
		.parent()
		.and_then(|parent| parent.parent())
		.expect("Could not get parent directory")
		.to_path_buf();

	let lib_path: PathBuf = lib_path
		.join("release")
		.exists()
		.then(|| lib_path.join("release"))
		.unwrap_or_else(|| lib_path.join("debug"))
		.join(extension_lib_name("commonext"));

	let extensions =
		ExtensionCollection::load_extensions(&[lib_path], true).expect("Failed to load extension");

	extensions
		.vfs_formats()
		.initialise_all(&testdir.join(""))
		.expect("Failed to initialise VFS roots");

	assert!(extensions.vfs_formats().exists("file1"));
	assert!(extensions.vfs_formats().is_file("file1"));
	assert!(!extensions.vfs_formats().is_directory("file1"));

	assert_eq!(
		extensions.vfs_formats().stat("file1").unwrap(),
		VfsFileStatResult {
			parent_path: "".to_owned(),
			name: "file1".to_owned(),
			is_directory: false,
			file_size: 14
		}
	);

	assert_eq!(
		extensions
			.vfs_formats()
			.load_file("file1")
			.unwrap()
			.as_slice(),
		b"This is file 1"
	);

	assert!(extensions.vfs_formats().exists("file2"));
	assert!(extensions.vfs_formats().is_file("file2"));
	assert!(!extensions.vfs_formats().is_directory("file2"));

	assert_eq!(
		extensions.vfs_formats().stat("file2").unwrap(),
		VfsFileStatResult {
			parent_path: "".to_owned(),
			name: "file2".to_owned(),
			is_directory: false,
			file_size: 14
		}
	);

	assert_eq!(
		extensions
			.vfs_formats()
			.load_file("file2")
			.unwrap()
			.as_slice(),
		b"This is file 2"
	);

	assert!(extensions.vfs_formats().exists("subdir/file3"));
	assert!(extensions.vfs_formats().is_file("subdir/file3"));
	assert!(!extensions.vfs_formats().is_directory("subdir/file3"));

	assert_eq!(
		extensions.vfs_formats().stat("subdir/file3").unwrap(),
		VfsFileStatResult {
			parent_path: "subdir".to_owned(),
			name: "file3".to_owned(),
			is_directory: false,
			file_size: 36
		}
	);

	assert_eq!(
		extensions
			.vfs_formats()
			.load_file("subdir/file3")
			.unwrap()
			.as_slice(),
		b"This is file 3, which is in a subdir"
	);
}
