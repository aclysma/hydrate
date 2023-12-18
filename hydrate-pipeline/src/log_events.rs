use hydrate_base::AssetId;
use crate::JobId;

#[derive(Debug, Copy, Clone)]
pub enum LogEventLevel {
    Warning,
    Error
}

pub struct LogEvent {
    pub asset_id: Option<AssetId>,
    pub job_id: Option<JobId>,
    pub level: LogEventLevel,
    pub message: String,
}