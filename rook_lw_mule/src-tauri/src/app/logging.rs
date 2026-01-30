use tracing::debug;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use std::path::Path;

use crate::RookLWMuleResult;

pub fn init_logging() -> RookLWMuleResult<WorkerGuard> {
    let log_dir = Path::new("logs");
    std::fs::create_dir_all(log_dir)?;
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_writer(non_blocking.and(std::io::stdout))
        .with_env_filter(env_filter)
        .init();

    debug!("Logging initialized.");

    // return the guard to flush logs.
    Ok(_guard)
}