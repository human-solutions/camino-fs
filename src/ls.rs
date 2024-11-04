use std::{collections::VecDeque, io};

use camino::{Utf8Path, Utf8PathBuf};

enum LsFilter {
    All,
    Files,
    Dirs,
}

pub struct Ls {
    recurse_if_fn: Box<dyn Fn(&Utf8Path) -> bool>,
    relative_paths: bool,
    path: Utf8PathBuf,
    filter: LsFilter,
    initialized: bool,
    entries: VecDeque<Utf8PathBuf>,
}

impl Ls {
    pub fn new(path: Utf8PathBuf) -> Self {
        Self {
            recurse_if_fn: Box::new(|_| false),
            relative_paths: false,
            path,
            filter: LsFilter::All,
            initialized: false,
            entries: VecDeque::new(),
        }
    }

    /// If true, the iterator returns relative paths instead of absolute paths.
    ///
    /// This is especially useful for copying or moving files.
    pub fn relative_paths(mut self) -> Self {
        self.relative_paths = true;
        self
    }

    /// Only recurse into directories that satisfy the given predicate, which is
    /// given a path that is always **relative** to the base path. In other words
    /// this is not changed by the _.relative_paths()_ function/setting.
    pub fn recurse_if<P: Fn(&Utf8Path) -> bool + 'static>(mut self, predicate: P) -> Self {
        self.recurse_if_fn = Box::new(predicate);
        self
    }

    /// Recurse into all directories.
    pub fn recurse(self) -> Self {
        self.recurse_if(|_| true)
    }

    /// Only return files
    pub fn files(self) -> Self {
        Self {
            filter: LsFilter::Files,
            ..self
        }
    }

    /// Only return directories
    pub fn dirs(self) -> Self {
        Self {
            filter: LsFilter::Dirs,
            ..self
        }
    }

    /// An iterator where you have to handle errors yourself.
    ///
    /// Set all options before calling this function.
    pub fn try_iter(self) -> TryLsIter {
        TryLsIter::new(self)
    }

    fn add_dir_entries(entries: &mut VecDeque<Utf8PathBuf>, dir: &Utf8Path) {
        let Ok(new_entries) = dir.read_dir_utf8() else {
            return;
        };

        entries.extend(new_entries.filter_map(|e| e.ok().map(|e| e.into_path())))
    }
}

impl Iterator for Ls {
    type Item = Utf8PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.initialized {
            Self::add_dir_entries(&mut self.entries, &self.path);
            self.initialized = true;
        }

        while let Some(mut path) = self.entries.pop_front() {
            let rel_path = path.strip_prefix(&self.path).unwrap();

            if path.is_dir() && (self.recurse_if_fn)(rel_path) {
                Self::add_dir_entries(&mut self.entries, &path);
            }
            if self.relative_paths {
                path = rel_path.to_path_buf();
            }
            match self.filter {
                LsFilter::All => return Some(path),
                LsFilter::Files if path.is_file() => return Some(path),
                LsFilter::Dirs if path.is_dir() => return Some(path),
                _ => {}
            }
        }
        None
    }
}

pub struct TryLsIter {
    ls: Ls,
    initialized: bool,
    entries: VecDeque<Utf8PathBuf>,
}

impl TryLsIter {
    fn new(ls: Ls) -> Self {
        Self {
            ls,
            initialized: false,
            entries: Default::default(),
        }
    }

    fn add_dir_entries(entries: &mut VecDeque<Utf8PathBuf>, dir: &Utf8Path) -> io::Result<()> {
        for entry in dir.read_dir_utf8()? {
            entries.push_back(entry?.into_path());
        }
        Ok(())
    }

    fn try_next_unfiltered(&mut self) -> io::Result<Option<Utf8PathBuf>> {
        if !self.initialized {
            Self::add_dir_entries(&mut self.entries, &self.ls.path)?;
            self.initialized = true;
        }
        while let Some(mut path) = self.entries.pop_front() {
            let rel_path = path.strip_prefix(&self.ls.path).unwrap();

            if path.is_dir() && (self.ls.recurse_if_fn)(&rel_path) {
                Self::add_dir_entries(&mut self.entries, &path)?;
            }
            if self.ls.relative_paths {
                path = rel_path.to_path_buf();
            }
            return Ok(Some(path));
        }
        Ok(None)
    }
}

impl Iterator for TryLsIter {
    type Item = io::Result<Utf8PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next_unfiltered() {
            Ok(Some(path)) => match self.ls.filter {
                LsFilter::All => Some(Ok(path)),
                LsFilter::Files if path.is_file() => Some(Ok(path)),
                LsFilter::Dirs if path.is_dir() => Some(Ok(path)),
                _ => None,
            },
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
