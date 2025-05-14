// Copyright (C) 2025 Kylin Soft. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use slog::{Drain, Level, Logger};
use slog::{OwnedKVList, Record};
use slog_async;
use slog_scope::set_global_logger;
use slog_term;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::sync::atomic::{AtomicUsize, Ordering};

static CURRENT_LEVEL: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(Level::Info.as_usize()));

struct DynamicLevelFilter<D> {
    drain: D,
}

impl<D> Drain for DynamicLevelFilter<D>
where
    D: Drain,
{
    type Ok = ();
    type Err = slog::Never;

    fn log(&self, record: &Record, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let current_level =
            Level::from_usize(CURRENT_LEVEL.load(Ordering::Relaxed)).unwrap_or(Level::Trace);
        if record.level().is_at_least(current_level) {
            let _ = self.drain.log(record, values);
        }
        Ok(())
    }
}

pub fn setup_logging(
    log_file: &Option<String>,
    log_level: Level,
) -> Result<slog_scope::GlobalLoggerGuard> {
	CURRENT_LEVEL.store(log_level.as_usize(), Ordering::Relaxed);

    let drain = if let Some(f) = log_file {
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(f)
            .map_err(|e| anyhow!("Open log file {} fail: {}", f, e))?;
        let buffered = BufWriter::new(log_file);
        let decorator = slog_term::PlainDecorator::new(buffered);
        let drain = slog_term::CompactFormat::new(decorator)
            .build()
            .fuse();
        slog_async::Async::new(drain).build().fuse()
    } else {
        let decorator = slog_term::TermDecorator::new().stderr().build();
        let drain = slog_term::CompactFormat::new(decorator)
            .build()
            .fuse();
        slog_async::Async::new(drain).build().fuse()
    };

    let filtered_drain = DynamicLevelFilter { drain };

    let logger = Logger::root(filtered_drain, slog::o!());
    Ok(set_global_logger(logger))
}

pub async fn set_log_level(str_level: &str) -> Result<()> {
	let level = parse_slog_level(str_level)?;

    CURRENT_LEVEL.store(level.as_usize(), Ordering::Relaxed);

	Ok(())
}

pub fn parse_slog_level(src: &str) -> Result<Level> {
    match src.to_lowercase().as_str() {
        "trace" => Ok(Level::Trace),
        "debug" => Ok(Level::Debug),
        "info" => Ok(Level::Info),
        "warning" => Ok(Level::Warning),
        "warn" => Ok(Level::Warning),
        "error" => Ok(Level::Error),
		"critical" => Ok(Level::Critical),
        _ => Err(anyhow!("Invalid log level: {}", src)),
    }
}
