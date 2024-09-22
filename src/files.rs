use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use eyre::ContextCompat;
use tracing::instrument;

pub type FdIdx = usize;
pub const NO_FILE_OPEN: FdIdx = FdIdx::MAX;

#[derive(Default)]
pub struct ProjectFileCache {
    file_names: HashSet<PathBuf>,
    file_readers: HashMap<PathBuf, BufReader<File>>,
}

impl ProjectFileCache {
    pub fn new(absolute_dir_path: &Path) -> Result<Self> {
        let mut ret = Self::default();

        // if absolute_dir_path.is_relative() {
        //     return Err(AppError::NotAbsolutePath);
        // }

        for f in walkdir::WalkDir::new(absolute_dir_path) {
            let f = f?;

            if f.file_type().is_file() {
                ret.file_names.insert(f.into_path());
            }
        }

        Ok(ret)
    }

    pub fn contains(&self, file_name: &Path) -> bool {
        self.file_names.contains(file_name)
    }

    pub fn file(&mut self, file_name: &Path) -> std::io::Result<&mut BufReader<File>> {
        if !self.file_readers.contains_key(file_name) {
            let file = File::open(file_name)?;
            let file = BufReader::new(file);
            self.file_readers.insert(file_name.to_owned(), file);
        }

        Ok(self.file_readers.get_mut(file_name).unwrap())
    }

    pub fn to_buffer(&self, buffer: &mut String) {
        use std::fmt::Write;

        buffer.clear();

        for f in self.file_names.iter() {
            if let Some(s) = f.as_os_str().to_str() {
                writeln!(buffer, "{s}");
            }
        }
    }
}
