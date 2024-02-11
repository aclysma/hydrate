use crate::build::JobRequestor;
use crate::JobId;
use hydrate_base::hashing::{HashMap, HashSet};
use hydrate_base::AssetId;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub enum LogEventLevel {
    Warning,
    Error,
    FatalError,
}

pub struct ImportLogEvent {
    pub path: PathBuf,
    pub asset_id: Option<AssetId>,
    pub level: LogEventLevel,
    pub message: String,
}

pub enum LogDataRef<'a> {
    Import(&'a ImportLogData),
    Build(&'a BuildLogData),
    None,
}

pub enum LogData {
    Import(Arc<ImportLogData>),
    Build(Arc<BuildLogData>),
}

impl LogData {
    pub fn id(&self) -> Uuid {
        match self {
            LogData::Import(x) => x.id,
            LogData::Build(x) => x.id,
        }
    }

    pub fn is_import(&self) -> bool {
        match self {
            LogData::Import(_) => true,
            _ => false,
        }
    }

    pub fn is_build(&self) -> bool {
        match self {
            LogData::Build(_) => true,
            _ => false,
        }
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        match self {
            LogData::Import(x) => x
                .end_instant
                .map(|end_instant| end_instant - x.start_instant),
            LogData::Build(x) => x
                .end_instant
                .map(|end_instant| end_instant - x.start_instant),
        }
    }

    pub fn start_time(&self) -> std::time::SystemTime {
        match self {
            LogData::Import(x) => x.start_time,
            LogData::Build(x) => x.start_time,
        }
    }
}

pub struct ImportLogData {
    pub(crate) id: Uuid,
    pub(crate) start_instant: std::time::Instant,
    pub(crate) end_instant: Option<std::time::Instant>,
    pub(crate) start_time: std::time::SystemTime,
    pub(crate) end_time: Option<std::time::SystemTime>,
    pub log_events: Vec<ImportLogEvent>,
}

impl ImportLogData {
    pub fn log_events(&self) -> &[ImportLogEvent] {
        &self.log_events
    }
}

impl Default for ImportLogData {
    fn default() -> Self {
        ImportLogData {
            id: Uuid::new_v4(),
            start_instant: std::time::Instant::now(),
            end_instant: None,
            start_time: std::time::SystemTime::now(),
            end_time: None,
            log_events: vec![],
        }
    }
}

#[derive(Debug)]
pub struct BuildLogEvent {
    pub asset_id: Option<AssetId>,
    pub job_id: Option<JobId>,
    pub level: LogEventLevel,
    pub message: String,
}

pub struct BuildLogData {
    pub(crate) id: Uuid,
    pub(crate) start_instant: std::time::Instant,
    pub(crate) end_instant: Option<std::time::Instant>,
    pub(crate) start_time: std::time::SystemTime,
    pub(crate) end_time: Option<std::time::SystemTime>,
    pub(crate) log_events: Vec<BuildLogEvent>,
    pub(crate) requestors: HashMap<JobId, Vec<JobRequestor>>,
}

impl Default for BuildLogData {
    fn default() -> Self {
        BuildLogData {
            id: Uuid::new_v4(),
            start_instant: std::time::Instant::now(),
            end_instant: None,
            start_time: std::time::SystemTime::now(),
            end_time: None,
            log_events: vec![],
            requestors: Default::default(),
        }
    }
}

impl BuildLogData {
    pub fn log_events(&self) -> &[BuildLogEvent] {
        &self.log_events
    }

    pub fn assets_relying_on_job(
        &self,
        job_id: JobId,
    ) -> Vec<AssetId> {
        let mut assets = vec![];
        let checked_requestors = HashSet::<JobId>::default();
        let mut requestor_check_queue = vec![job_id];

        while let Some(requestor) = requestor_check_queue.pop() {
            for requestor in self.requestors.get(&requestor).unwrap() {
                match requestor {
                    JobRequestor::Builder(asset_id) => assets.push(*asset_id),
                    JobRequestor::Job(job_id) => {
                        if !checked_requestors.contains(job_id) {
                            requestor_check_queue.push(*job_id)
                        }
                    }
                }
            }
        }

        assets
    }
}
