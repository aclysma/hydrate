use notify::Watcher;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, UNIX_EPOCH},
};

use crate::error::{Error, Result};

pub fn canonicalize_path(path: &Path) -> PathBuf {
    use path_slash::{PathBufExt, PathExt};
    let cleaned_path = PathBuf::from_slash(path_clean::clean(&path.to_slash_lossy()));
    PathBuf::from(dunce::simplified(&cleaned_path))
}

// When dropped, sends an exit "error" message via the same channel we receive messages from notify
pub struct StopHandle {
    tx: mpsc::Sender<notify::DebouncedEvent>,
}

impl Drop for StopHandle {
    fn drop(&mut self) {
        let _ = self.tx.send(notify::DebouncedEvent::Error(
            notify::Error::Generic("EXIT".to_string()),
            Option::None,
        ));
    }
}

// Wraps the notify crate:
// - supports symlinks
// - encapsulates notify so other systems don't use it directly
// - generates update messages for all files on startup
/// The purpose of DirWatcher is to provide enough information to
/// determine which files may be candidates for going through the asset import process.
/// It handles updating watches for directories behind symlinks and scans directories on create/delete.
pub struct DirWatcher {
    // the watcher object from the notify crate
    watcher: notify::RecommendedWatcher,

    // Symlinks we have discovered and where they point
    symlink_map: HashMap<PathBuf, PathBuf>,

    // Refcount of all dirs being watched. We call unwatch on the watcher when it hits 0
    watch_refs: HashMap<PathBuf, i32>,

    // All dirs being watched
    dirs: Vec<PathBuf>,

    // Channel for events from the watcher (and we insert events ourselves to indicate shutdown)
    // TODO: Switch to crossbeam when upgrading to notify 5.x
    rx: mpsc::Receiver<notify::DebouncedEvent>,
    tx: mpsc::Sender<notify::DebouncedEvent>,

    // The final output of this struct goes here
    asset_tx: crossbeam_channel::Sender<FileEvent>,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_type: fs::FileType,
    pub last_modified: u64,
    pub length: u64,
}
#[derive(Debug)]
pub enum FileEvent {
    Updated(PathBuf, FileMetadata),
    Renamed(PathBuf, PathBuf, FileMetadata),
    Removed(PathBuf),
    FileError(Error),
    // ScanStart is called when a directory is about to be scanned.
    // Scanning can be recursive
    ScanStart(PathBuf),
    // ScanEnd indicates the end of a scan. The set of all watched directories is also sent
    ScanEnd(PathBuf, Vec<PathBuf>),
    // Watch indicates that a new directory is being watched, probably due to a symlink being created
    Watch(PathBuf),
    // Unwatch indicates that a new directory has stopped being watched, probably due to a symlink being removed
    Unwatch(PathBuf),
}

// Sanitizes file metadata
pub(crate) fn file_metadata(metadata: &fs::Metadata) -> FileMetadata {
    let modify_time = metadata.modified().unwrap_or(UNIX_EPOCH);
    let since_epoch = modify_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_epoch.as_secs() * 1000 + u64::from(since_epoch.subsec_nanos()) / 1_000_000;
    FileMetadata {
        file_type: metadata.file_type(),
        length: metadata.len(),
        last_modified: in_ms,
    }
}

impl DirWatcher {
    // Starts a watcher running on the given paths
    pub fn from_path_iter<'a, T>(
        paths: T,
        chan: crossbeam_channel::Sender<FileEvent>,
    ) -> Result<DirWatcher>
    where
        T: IntoIterator<Item = &'a Path>,
    {
        let (tx, rx) = mpsc::channel();
        let mut asset_watcher = DirWatcher {
            watcher: notify::watcher(tx.clone(), Duration::from_millis(300))?,
            symlink_map: HashMap::new(),
            watch_refs: HashMap::new(),
            dirs: Vec::new(),
            rx,
            tx,
            asset_tx: chan,
        };

        for path in paths {
            let path = PathBuf::from(path);
            let path = if path.is_relative() {
                std::env::current_dir()?.join(path)
            } else {
                path
            };
            let path = canonicalize_path(&path);
            asset_watcher.watch(&path)?;
        }

        Ok(asset_watcher)
    }

    // Returns a StopHandle that will halt watching when it's dropped
    pub fn stop_handle(&self) -> StopHandle {
        StopHandle {
            tx: self.tx.clone(),
        }
    }

    // Visit all files, call handle_notify_event() passing the event created by the provided callback
    fn scan_directory<F>(
        &mut self,
        dir: &Path,
        evt_create: &F,
    ) -> Result<()>
    where
        F: Fn(PathBuf) -> notify::DebouncedEvent,
    {
        let canonical_dir = canonicalize_path(dir);
        self.asset_tx
            .send(FileEvent::ScanStart(canonical_dir.clone()))
            .map_err(|_| Error::SendError)?;
        let result = self.scan_directory_recurse(&canonical_dir, evt_create);
        self.asset_tx
            .send(FileEvent::ScanEnd(canonical_dir, self.dirs.clone()))
            .map_err(|_| Error::SendError)?;
        result
    }

    fn scan_directory_recurse<F>(
        &mut self,
        dir: &Path,
        evt_create: &F,
    ) -> Result<()>
    where
        F: Fn(PathBuf) -> notify::DebouncedEvent,
    {
        match fs::read_dir(dir) {
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::IO(e)),
            Ok(dir_entry) => {
                for entry in dir_entry {
                    match entry {
                        Err(ref e) if e.kind() == io::ErrorKind::NotFound => continue,
                        Err(e) => return Err(Error::IO(e)),
                        Ok(entry) => {
                            let evt = self.handle_notify_event(evt_create(entry.path()), true)?;
                            if let Some(evt) = evt {
                                self.asset_tx.send(evt).map_err(|_| Error::SendError)?;
                            }
                            let metadata;
                            match entry.metadata() {
                                Err(ref e) if e.kind() == io::ErrorKind::NotFound => continue,
                                Err(e) => return Err(Error::IO(e)),
                                Ok(m) => metadata = m,
                            }
                            let is_dir = metadata.is_dir();
                            if is_dir {
                                self.scan_directory_recurse(&entry.path(), evt_create)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) {
        // Emit a Create for every path recursively
        for dir in &self.dirs.clone() {
            if let Err(err) = self.scan_directory(dir, &|path| notify::DebouncedEvent::Create(path))
            {
                self.asset_tx
                    .send(FileEvent::FileError(err))
                    .expect("Failed to send file error event. Ironic...");
            }
        }

        loop {
            match self.rx.recv() {
                Ok(event) => match self.handle_notify_event(event, false) {
                    // By default, just proxy the event into the channel
                    Ok(maybe_event) => {
                        if let Some(evt) = maybe_event {
                            log::debug!("File event: {:?}", evt);
                            self.asset_tx.send(evt).unwrap();
                        }
                    }
                    Err(err) => match err {
                        // If notify cannot assure correct modification events (sometimes due to overflowing a queue,
                        // may be a limit on the OS) we have to reset to good state bu scanning everything
                        Error::RescanRequired => {
                            for dir in &self.dirs.clone() {
                                if let Err(err) = self.scan_directory(dir, &|path| {
                                    notify::DebouncedEvent::Create(path)
                                }) {
                                    self.asset_tx
                                        .send(FileEvent::FileError(err))
                                        .expect("Failed to send file error event");
                                }
                            }
                        }
                        Error::Exit => break,
                        _ => self
                            .asset_tx
                            .send(FileEvent::FileError(err))
                            .expect("Failed to send file error event"),
                    },
                },
                Err(_) => {
                    self.asset_tx
                        .send(FileEvent::FileError(Error::RecvError))
                        .expect("Failed to send file error event");

                    return;
                }
            }
        }
    }

    fn watch(
        &mut self,
        path: &Path,
    ) -> Result<bool> {
        let refs = *self.watch_refs.get(path).unwrap_or(&0);
        match refs {
            0 => {
                self.watcher.watch(path, notify::RecursiveMode::Recursive)?;
                self.dirs.push(path.to_path_buf());
                self.watch_refs.insert(path.to_path_buf(), 1);
                return Ok(true);
            }
            refs if refs > 0 => {
                self.watch_refs
                    .entry(path.to_path_buf())
                    .and_modify(|r| *r += 1);
            }
            _ => {}
        }
        Ok(false)
    }

    fn unwatch(
        &mut self,
        path: &Path,
    ) -> Result<bool> {
        let refs = *self.watch_refs.get(path).unwrap_or(&0);
        if refs == 1 {
            self.watcher.unwatch(path)?;
            for i in 0..self.dirs.len() {
                if *path == self.dirs[i] {
                    self.dirs.remove(i);
                    break;
                }
            }
            self.watch_refs.remove(path);
            return Ok(true);
        } else if refs > 0 {
            self.watch_refs
                .entry(path.to_path_buf())
                .and_modify(|r| *r -= 1);
        }
        Ok(false)
    }

    // Called when we receive about changes, most of the time we will detect that the path is not
    // a symlink and do nothing
    fn handle_updated_symlink(
        &mut self,
        src: Option<&PathBuf>,
        dst: Option<&PathBuf>,
    ) -> Result<bool> {
        let mut recognized_symlink = false;
        if let Some(src) = src {
            if self.symlink_map.contains_key(src) {
                // This was previous a symlink. Remove the watch, and maybe clean up the watch
                let to_unwatch = self.symlink_map[src].clone();
                match self.unwatch(&to_unwatch) {
                    Ok(unwatched) => {
                        if unwatched {
                            self.scan_directory(&to_unwatch, &|p| {
                                notify::DebouncedEvent::Remove(p)
                            })?;
                        }
                        self.symlink_map.remove(src);
                        self.asset_tx
                            .send(FileEvent::Unwatch(to_unwatch))
                            .map_err(|_| Error::SendError)?;
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
                recognized_symlink = true;
            }
        }
        if let Some(dst) = dst {
            let link = fs::read_link(&dst);
            if let Ok(link_path) = link {
                let link_path = canonicalize_path(&dst.join(link_path));
                match self.watch(&link_path) {
                    Ok(watched) => {
                        if watched {
                            self.scan_directory(&link_path, &|p| {
                                notify::DebouncedEvent::Create(p)
                            })?;
                        }
                        self.symlink_map.insert(dst.clone(), link_path.clone());
                        self.asset_tx
                            .send(FileEvent::Watch(link_path))
                            .map_err(|_| Error::SendError)?;
                    }
                    Err(Error::Notify(notify::Error::Generic(text)))
                        if text == "Input watch path is neither a file nor a directory." =>
                    {
                        // skip the symlink if it's not a valid path
                    }
                    Err(Error::Notify(notify::Error::PathNotFound)) => {}
                    Err(Error::IO(err)) | Err(Error::Notify(notify::Error::Io(err)))
                        if err.kind() == std::io::ErrorKind::NotFound =>
                    {
                        // skip the symlink if it no longer exists or can't be watched
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
                recognized_symlink = true;
            }
        }
        Ok(recognized_symlink)
    }

    // Handles events from notify, and manually triggered from calling scan_directory()
    fn handle_notify_event(
        &mut self,
        event: notify::DebouncedEvent,
        is_scanning: bool,
    ) -> Result<Option<FileEvent>> {
        match event {
            notify::DebouncedEvent::Create(path) | notify::DebouncedEvent::Write(path) => {
                let path = canonicalize_path(&path);
                self.handle_updated_symlink(Option::None, Some(&path))?;

                // Try to get info on this file, following the symlink if necessary
                let metadata_result = match fs::metadata(&path) {
                    Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                    Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
                        log::warn!(
                            "Permission denied when fetching metadata for create event {:?}",
                            path,
                        );
                        Ok(None)
                    }
                    Err(e) => Err(Error::IO(e)),
                    Ok(metadata) => Ok(Some(FileEvent::Updated(
                        path.clone(),
                        file_metadata(&metadata),
                    ))),
                };

                // If metadata hit NotFound/PermissionDenied, get metadata without trying to follow symlink
                match metadata_result {
                    Ok(None) => {
                        if let Ok(metadata) = fs::symlink_metadata(&path) {
                            Ok(Some(FileEvent::Updated(path, file_metadata(&metadata))))
                        } else {
                            Ok(None)
                        }
                    }
                    result => result,
                }
            }
            notify::DebouncedEvent::Rename(src, dest) => {
                let src = canonicalize_path(&src);
                let dest = canonicalize_path(&dest);
                self.handle_updated_symlink(Some(&src), Some(&dest))?;
                match fs::metadata(&dest) {
                    Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                    Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
                        log::warn!(
                            "Permission denied when fetching metadata for rename event {:?}->{:?}",
                            src,
                            dest
                        );
                        Ok(None)
                    }
                    Err(e) => Err(Error::IO(e)),
                    Ok(metadata) => {
                        if metadata.is_dir() && !is_scanning {
                            self.scan_directory(&dest, &|p| {
                                let replaced = canonicalize_path(&src.join(
                                    p.strip_prefix(&dest).expect("Failed to strip prefix dir"),
                                ));
                                notify::DebouncedEvent::Rename(replaced, p)
                            })?;
                        }
                        Ok(Some(FileEvent::Renamed(
                            src,
                            dest,
                            file_metadata(&metadata),
                        )))
                    }
                }
            }
            notify::DebouncedEvent::Remove(path) => {
                let path = canonicalize_path(&path);
                self.handle_updated_symlink(Some(&path), Option::None)?;
                Ok(Some(FileEvent::Removed(path)))
            }
            notify::DebouncedEvent::Rescan => Err(Error::RescanRequired),
            notify::DebouncedEvent::Error(_, _) => Err(Error::Exit),
            _ => Ok(None),
        }
    }
}
