use std::{collections::VecDeque, io};

use camino::{Utf8Path, Utf8PathBuf};

enum LsFilter {
    All,
    Files,
    Dirs,
}

pub struct Ls {
    recurse_if_fn: Box<dyn Fn(&Utf8Path) -> bool>,
    path: Utf8PathBuf,
    filter: LsFilter,
}

impl Ls {
    pub fn new(path: Utf8PathBuf) -> Self {
        Self {
            recurse_if_fn: Box::new(|_| false),
            path,
            filter: LsFilter::All,
        }
    }

    pub fn recurse_if<P: Fn(&Utf8Path) -> bool + 'static>(mut self, predicate: P) -> Self {
        self.recurse_if_fn = Box::new(predicate);
        self
    }

    pub fn recurse(self) -> Self {
        self.recurse_if(|_| true)
    }

    pub fn files(self) -> Self {
        Self {
            filter: LsFilter::Files,
            ..self
        }
    }
    pub fn dirs(self) -> Self {
        Self {
            filter: LsFilter::Dirs,
            ..self
        }
    }

    pub fn try_iter(self) -> TryLsIter {
        TryLsIter::new(self)
    }
}

impl Iterator for Ls {
    type Item = Utf8PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
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
        while let Some(path) = self.entries.pop_front() {
            if path.is_dir() {
                if (self.ls.recurse_if_fn)(&path) {
                    Self::add_dir_entries(&mut self.entries, &path)?;
                }
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

pub struct LsIter {
    ls: Ls,
    initialized: bool,
    entries: VecDeque<Utf8PathBuf>,
}

impl LsIter {
    fn add_dir_entries(entries: &mut VecDeque<Utf8PathBuf>, dir: &Utf8Path) {
        let Ok(new_entries) = dir.read_dir_utf8() else {
            return;
        };

        entries.extend(new_entries.filter_map(|e| e.ok()).map(|e| e.into_path()))
    }
}

impl Iterator for LsIter {
    type Item = Utf8PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.initialized {
            Self::add_dir_entries(&mut self.entries, &self.ls.path);
            self.initialized = true;
        }

        while let Some(path) = self.entries.pop_front() {
            if path.is_dir() {
                if (self.ls.recurse_if_fn)(&path) {
                    Self::add_dir_entries(&mut self.entries, &path);
                }
            }
            match self.ls.filter {
                LsFilter::All => return Some(path),
                LsFilter::Files if path.is_file() => return Some(path),
                LsFilter::Dirs if path.is_dir() => return Some(path),
                _ => {}
            }
        }
        None
    }
}
