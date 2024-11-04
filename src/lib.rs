mod fs;
mod ls;

use fs::*;
use ls::Ls;
use std::{collections::VecDeque, io, iter, path::Path, time::SystemTime};

pub use camino::{Utf8Path, Utf8PathBuf};

pub trait Utf8PathBufExt {
    fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Utf8PathBuf>;
}

impl Utf8PathBufExt for Utf8PathBuf {
    fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Utf8PathBuf::from_path_buf(path.as_ref().to_path_buf()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Could not convert to pathbuf: {e:?}"),
            )
        })
    }
}

pub trait Utf8PathExt {
    /// Returns the path relative to the given base path.
    ///
    /// This is really just a wrapper around Utf8Path's `strip_prefix` method.
    fn relative_to<P: AsRef<Path>>(&self, path: P) -> Option<&'_ Utf8Path>;

    /// Add an extension to the path. If the path already has an extension, it is appended
    /// with the new extension.
    ///
    /// Example:
    ///
    /// ```
    /// use camino_fs::*;
    ///
    /// let path = Utf8Path::new("file.txt").join_ext("gz");
    /// assert_eq!(path.all_extensions(), Some("txt.gz"));
    /// ```
    fn join_ext<S: AsRef<str>>(&self, ext: S) -> Utf8PathBuf;

    /// Returns an iterator over the extensions of the path.
    fn extensions<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a>;

    /// Returns all extensions, i.e. the string after the first dot in the filename
    fn all_extensions(&self) -> Option<&str>;

    /// Returns an iterator over the entries in the directory (non recursively)
    /// or an error if the path is not a directory.
    ///
    /// If there is an error getting the path of an entry, it is skipped.
    ///
    /// Note that this is not performance optimized and may be slow for large directories.
    fn ls(&self) -> Ls;

    /// Create directory if it does not exist.
    fn mkdir(&self) -> io::Result<()>;

    /// Create all directories if they don't exist.
    fn mkdirs(&self) -> io::Result<()>;

    /// Remove the file or directory at the path.
    ///
    /// Does nothing if the path does not exist.
    fn rm(&self) -> io::Result<()>;

    /// Remove all files and directories in the directory recursively that match the predicate.
    fn rm_matching<P: Fn(&Utf8Path) -> bool>(&self, predicate: P) -> io::Result<()>;

    /// Copy recursively from the path to the destination path.
    fn cp<P: Into<Utf8PathBuf>>(&self, to: P) -> io::Result<()>;

    /// Renames a file or directory to a new name, replacing the original file if to already exists.
    fn mv<P: Into<Utf8PathBuf>>(&self, to: P) -> io::Result<()>;

    /// Throw an error if the path does not exist.
    fn assert_exists(&self) -> io::Result<()>;

    /// Throw an error if the path is not a directory.
    fn assert_dir(&self) -> io::Result<()>;

    /// Throw an error if the path is not a file.
    fn assert_file(&self) -> io::Result<()>;

    /// Write to the file at the path. Creates the file if it does not exist
    /// and replaces the content if it does.
    ///
    /// If the path also contains directories that do not exist, they will be created.
    fn write<B: AsRef<[u8]>>(&self, buf: B) -> io::Result<()>;

    /// Read a file
    fn read_bytes(&self) -> io::Result<Vec<u8>>;

    /// Read a file as a string
    fn read_string(&self) -> io::Result<String>;

    /// Get the system time for a file or folder
    fn mtime(&self) -> Option<SystemTime>;
}

impl Utf8PathExt for Utf8Path {
    fn assert_exists(&self) -> io::Result<()> {
        if !self.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path \"{}\" does not exist or you don't have access!", self),
            ));
        }
        Ok(())
    }

    fn assert_dir(&self) -> io::Result<()> {
        if !self.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path \"{}\" is not a directory!", self),
            ));
        }
        Ok(())
    }

    fn assert_file(&self) -> io::Result<()> {
        if !self.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path \"{}\" is not a file!", self),
            ));
        }
        Ok(())
    }

    fn cp<P: Into<Utf8PathBuf>>(&self, to: P) -> io::Result<()> {
        self.assert_exists()?;
        let dest = to.into();

        if self.is_dir() {
            self.assert_dir()?;

            dest.mkdirs()?;

            let mut entries: VecDeque<Utf8PathBuf> = self.ls().collect();

            while let Some(src_path) = entries.pop_front() {
                let rel_path = src_path.strip_prefix(self).unwrap();
                let dest_path = dest.join(rel_path);

                if src_path.is_dir() {
                    entries.extend(src_path.ls());
                    dest_path.mkdir()?;
                } else {
                    fs_copy(&src_path, &dest_path)?;
                }
            }
        } else {
            fs_copy(self, &dest)?;
        }
        Ok(())
    }

    fn mv<P: Into<Utf8PathBuf>>(&self, to: P) -> io::Result<()> {
        self.assert_exists()?;
        fs_rename(self, &to.into())
    }

    fn rm(&self) -> io::Result<()> {
        if !self.exists() {
            Ok(())
        } else if self.is_dir() {
            fs_remove_dir_all(self)
        } else {
            fs_remove_file(self)
        }
    }

    fn rm_matching<P: Fn(&Utf8Path) -> bool>(&self, predicate: P) -> io::Result<()> {
        if self.is_dir() {
            for file in self.ls().filter(|p| predicate(p)) {
                file.rm()?;
            }
        } else if predicate(self) {
            self.rm()?;
        }
        Ok(())
    }

    fn mkdir(&self) -> io::Result<()> {
        if !self.exists() {
            fs_create_dir(self)?;
        }
        Ok(())
    }

    fn mkdirs(&self) -> io::Result<()> {
        fs_create_dir_all(self)
    }

    fn ls(&self) -> Ls {
        Ls::new(self.to_path_buf())
    }

    fn extensions<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        if let Some(name) = self.file_name() {
            Box::new(name.split('.').take(1))
        } else {
            Box::new(iter::empty())
        }
    }

    fn join_ext<S: AsRef<str>>(&self, ext: S) -> Utf8PathBuf {
        let ext = ext.as_ref();
        let mut s = self.to_string();
        if !ext.starts_with('.') {
            s.push('.');
        }
        s.push_str(ext);
        Utf8PathBuf::from(s)
    }

    fn all_extensions(&self) -> Option<&str> {
        Some(self.file_name()?.split_once('.')?.1)
    }

    fn relative_to<P: AsRef<Path>>(&self, path: P) -> Option<&'_ Utf8Path> {
        self.strip_prefix(path).ok()
    }

    fn write<B: AsRef<[u8]>>(&self, buf: B) -> io::Result<()> {
        if let Some(parent) = self.parent() {
            parent.mkdirs()?;
        }
        fs_write(self, buf.as_ref())
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        fs_read(self)
    }

    fn read_string(&self) -> io::Result<String> {
        fs_read_to_string(self)
    }

    fn mtime(&self) -> Option<SystemTime> {
        self.metadata().ok().map(|md| md.modified().unwrap())
    }
}
