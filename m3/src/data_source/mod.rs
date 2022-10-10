use std::path::{Path, PathBuf};
use nexdb::FileSystemDataSource;

pub struct FileSystemPackage {
    root_path: PathBuf,
    data_source: Option<nexdb::FileSystemDataSource>,
}

impl FileSystemPackage {
    pub fn new<T: Into<PathBuf>>(root_path: T) -> Self {
        FileSystemPackage {
            root_path: root_path.into(),
            data_source: None
        }
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn load(&mut self, db: &mut nexdb::Database) {
        log::info!(
            "Adding schema source dir {:?}",
            self.root_path
        );

        let ds = nexdb::FileSystemDataSource::new(&self.root_path, db);

        self.data_source = Some(ds);
    }

    pub fn unload(&self) {

    }

    pub fn data_source(&self) -> Option<&FileSystemDataSource> {
        self.data_source.as_ref()
    }

    pub fn data_source_mut(&mut self) -> Option<&mut FileSystemDataSource> {
        self.data_source.as_mut()
    }
}
