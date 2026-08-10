use std::fs;
use std::path::Path;

use crate::extensions::ExtensionCollection;
use target_test_dir::with_test_dir;

#[test]
#[with_test_dir]
fn stat()
{
	let testdir = get_test_dir!();
	create_test_filesystem(testdir.as_path());

	let extensions = ExtensionCollection::load_extensions(&[testdir.join("nonexistent.lib")], true);
	println!("{}", extensions.err().unwrap());
	todo!();
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
