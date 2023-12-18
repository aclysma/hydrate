use std::path::PathBuf;
use hydrate_base::AssetId;
use crate::JobId;

#[derive(Debug, Copy, Clone)]
pub enum LogEventLevel {
    Warning,
    Error
}

pub struct ImportLogEvent {
    pub path: PathBuf,
    pub asset_id: Option<AssetId>,
    pub level: LogEventLevel,
    pub message: String,
}

pub struct BuildLogEvent {
    pub asset_id: Option<AssetId>,
    pub job_id: Option<JobId>,
    pub level: LogEventLevel,
    pub message: String,
}