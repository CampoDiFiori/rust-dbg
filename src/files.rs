use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use color_eyre::Result;

#[derive(Default)]
pub struct ProjectFileCache {
    inner: HashSet<PathBuf>,
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
                ret.inner.insert(f.into_path());
            }
        }

        Ok(ret)
    }

    pub fn contains(&self, file_name: &Path) -> bool {
        self.inner.contains(file_name)
    }
}
