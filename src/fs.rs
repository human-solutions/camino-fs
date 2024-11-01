use std::io;

use camino::Utf8Path;

/// Copies the contents of one file to another. This function will also copy the
/// permission bits of the original file to the destination file.
///
/// Wrapper for [`fs::copy`](https://doc.rust-lang.org/stable/std/fs/fn.copy.html).
pub fn fs_copy(from: &Utf8Path, to: &Utf8Path) -> io::Result<()> {
    std::fs::copy(from, to)
        .map_err(|e| io::Error::other(format!("Could not copy {from} to {to} due to: {e}")))
        .map(|_| ())
}

/// Rename a file or directory to a new name, replacing the original file if to already exists.
///
/// Wrapper for [`fs::rename`](https://doc.rust-lang.org/stable/std/fs/fn.rename.html).
pub fn fs_rename(from: &Utf8Path, to: &Utf8Path) -> io::Result<()> {
    std::fs::rename(from, to)
        .map_err(|e| io::Error::other(format!("Could not rename {from} to {to} due to: {e}")))
        .map(|_| ())
}

/// Removes a directory at this path, after removing all its contents. Use carefully!
///
/// Wrapper for [`fs::remove_dir_all`](https://doc.rust-lang.org/stable/std/fs/fn.remove_dir_all.html).
pub fn fs_remove_dir_all(path: &Utf8Path) -> io::Result<()> {
    std::fs::remove_dir_all(path)
        .map_err(|e| io::Error::other(format!("Could not remove {path} due to: {e}")))
        .map(|_| ())
}

/// Removes a file from the filesystem.
///
/// Wrapper for [`fs::remove_file`](https://doc.rust-lang.org/stable/std/fs/fn.remove_file.html).
pub fn fs_remove_file(path: &Utf8Path) -> io::Result<()> {
    std::fs::remove_file(path)
        .map_err(|e| io::Error::other(format!("Could not remove {path} due to: {e}")))
        .map(|_| ())
}

/// Creates a new, empty directory at the provided path.
///
/// Wrapper for [`fs::create_dir`](https://doc.rust-lang.org/stable/std/fs/fn.create_dir.html).
pub fn fs_create_dir(path: &Utf8Path) -> io::Result<()> {
    std::fs::create_dir(path)
        .map_err(|e| io::Error::other(format!("Could not create directory {path} due to: {e}")))
}

/// Recursively create a directory and all of its parent components if they are missing.
///
/// Wrapper for [`fs::create_dir_all`](https://doc.rust-lang.org/stable/std/fs/fn.create_dir_all.html).
pub fn fs_create_dir_all(path: &Utf8Path) -> io::Result<()> {
    std::fs::create_dir_all(path).map_err(|e| {
        io::Error::other(format!(
            "Could not create directories for {path} due to: {e}"
        ))
    })
}

/// Read the entire contents of a file into a bytes vector.
///
/// Wrapper for [`fs::read`](https://doc.rust-lang.org/stable/std/fs/fn.read.html).
pub fn fs_read(path: &Utf8Path) -> io::Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| io::Error::other(format!("Could not read {path} due to: {e}")))
}

/// Read the entire contents of a file into a string.
///
/// Wrapper for [`fs::read_to_string`](https://doc.rust-lang.org/stable/std/fs/fn.read_to_string.html).
pub fn fs_read_to_string(path: &Utf8Path) -> io::Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| io::Error::other(format!("Could not read {path} due to: {e}")))
}

/// Write a slice as the entire contents of a file.
///
/// Wrapper for [`fs::write`](https://doc.rust-lang.org/stable/std/fs/fn.write.html).
pub fn fs_write(path: &Utf8Path, bytes: &[u8]) -> io::Result<()> {
    std::fs::write(path, bytes)
        .map_err(|e| io::Error::other(format!("Could not write to {path} due to: {e}")))
}
