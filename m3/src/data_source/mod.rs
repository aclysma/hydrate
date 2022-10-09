use std::path::{Path, PathBuf};

pub struct LoadedFile {
    path: PathBuf
}

impl LoadedFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub struct FileSystemLoadedState {
    files: Vec<LoadedFile>
}

impl FileSystemLoadedState {
    pub fn files(&self) -> &[LoadedFile] {
        &self.files
    }
}


pub struct FileSystemDataSource {
    root_path: PathBuf,
    loaded_state: Option<FileSystemLoadedState>,

}

impl FileSystemDataSource {
    pub fn new<T: Into<PathBuf>>(root_path: T) -> Self {
        FileSystemDataSource {
            root_path: root_path.into(),
            loaded_state: None
        }
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn load(&mut self) {
        log::info!(
            "Adding schema source dir {:?}",
            self.root_path
        );

        let mut loaded_state = FileSystemLoadedState {
            files: Default::default()
        };

        let walker = globwalk::GlobWalkerBuilder::new(&self.root_path, "*")
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file_path in walker {
            println!("path {:?}", file_path);
            let file = file_path.unwrap();
            //file.
            let file = LoadedFile {
                path: file.path().to_path_buf()
            };

            loaded_state.files.push(file);
        }

        self.loaded_state = Some(loaded_state);
    }

    pub fn unload(&self) {

    }

    pub fn loaded_state(&self) -> Option<&FileSystemLoadedState> {
        self.loaded_state.as_ref()
    }

    pub fn loaded_state_mut(&mut self) -> Option<&mut FileSystemLoadedState> {
        self.loaded_state.as_mut()
    }
}
