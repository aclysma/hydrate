use std::path::{Path, PathBuf};


pub struct FileSystemLoadedState {

}



pub struct FileSystemDataSource {
    root_path: PathBuf,

}

impl FileSystemDataSource {
    pub fn new<T: Into<PathBuf>>(root_path: T) -> Self {
        FileSystemDataSource {
            root_path: root_path.into()
        }
    }

    pub fn load(&self) {

        log::info!(
            "Adding schema source dir {:?}",
            self.root_path
        );

        let walker = globwalk::GlobWalkerBuilder::new(&self.root_path, "*")
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            println!("path {:?}", file);
        }
    }

    pub fn unload(&self) {

    }
}
