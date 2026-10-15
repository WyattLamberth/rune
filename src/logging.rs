//! CLI diagnostic adapter: one synchronous sink for backend and final errors.
use anyhow::{Context, Result};
use clap::ValueEnum;
use std::fmt::Debug;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context as LayerContext, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}

struct Output {
    writer: Box<dyn Write + Send>,
    failure: Option<String>,
}

#[derive(Clone)]
pub struct LogHandle(Arc<Mutex<Output>>);

impl LogHandle {
    fn new(writer: impl Write + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(Output {
            writer: Box::new(writer),
            failure: None,
        })))
    }

    fn write(&self, line: &[u8]) {
        let mut output = self.0.lock().unwrap_or_else(|error| error.into_inner());
        // Hold the lock across the complete record and flush. There is no
        // buffered guard to lose when the runtime returns an exit status.
        if let Err(error) = output
            .writer
            .write_all(line)
            .and_then(|()| output.writer.flush())
        {
            output.failure.get_or_insert_with(|| error.to_string());
        }
    }

    pub fn failure(&self) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .failure
            .clone()
    }
}

#[derive(Default)]
struct Fields {
    message: String,
    values: serde_json::Map<String, serde_json::Value>,
}

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.values.insert(field.name().to_owned(), value.into());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.record_str(field, &format!("{value:?}"));
    }
}

struct LogLayer {
    output: LogHandle,
    format: LogFormat,
}

impl<S: Subscriber> Layer<S> for LogLayer {
    fn on_event(&self, event: &Event<'_>, _context: LayerContext<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let level = event.metadata().level().as_str().to_ascii_lowercase();
        let time = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
        let line = match self.format {
            LogFormat::Json => {
                let mut record = serde_json::json!({
                    "level": level, "msg": fields.message, "time": time
                });
                if !fields.values.is_empty() {
                    record["fields"] = fields.values.into();
                }
                format!("{record}\n")
            }
            LogFormat::Text => {
                let extra = if fields.values.is_empty() {
                    String::new()
                } else {
                    format!(" fields={}", serde_json::Value::Object(fields.values))
                };
                // JSON quoting keeps embedded newlines inside a single record.
                format!(
                    "time={time:?} level={level} msg={:?}{extra}\n",
                    fields.message
                )
            }
        };
        self.output.write(line.as_bytes());
    }
}

fn subscriber(output: LogHandle, format: LogFormat, debug: bool) -> impl Subscriber + Send + Sync {
    Registry::default()
        .with(if debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        })
        .with(LogLayer { output, format })
}

pub fn init(path: Option<&Path>, format: LogFormat, debug: bool) -> Result<LogHandle> {
    let output = match path {
        Some(path) => LogHandle::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .context("cannot open requested log")?,
        ),
        None => LogHandle::new(std::io::stderr()),
    };
    subscriber(output.clone(), format, debug)
        .try_init()
        .context("cannot install diagnostic subscriber")?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);
    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn backend_and_final_events_share_format_and_level_filter() {
        for format in [LogFormat::Text, LogFormat::Json] {
            for debug in [false, true] {
                let buffer = Buffer::default();
                let output = LogHandle::new(buffer.clone());
                tracing::subscriber::with_default(
                    subscriber(output.clone(), format, debug),
                    || {
                        tracing::debug!("backend debug");
                        tracing::info!("backend info");
                        tracing::warn!(reason = "fixture", "backend warning\nsecond line");
                        tracing::error!("final error");
                    },
                );
                let text = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
                assert_eq!(text.lines().count(), if debug { 4 } else { 3 });
                assert_eq!(text.contains("backend debug"), debug);
                assert!(text.contains("backend info"));
                assert!(text.contains("backend warning\\nsecond line"));
                assert!(text.contains("final error"));
                if matches!(format, LogFormat::Json) {
                    for line in text.lines() {
                        let record: serde_json::Value = serde_json::from_str(line).unwrap();
                        assert!(record["level"].is_string());
                        assert!(record["msg"].is_string());
                        chrono::DateTime::parse_from_rfc3339(record["time"].as_str().unwrap())
                            .unwrap();
                    }
                }
                assert!(output.failure().is_none());
            }
        }
    }

    #[test]
    fn concurrent_events_are_complete_json_records() {
        let buffer = Buffer::default();
        let output = LogHandle::new(buffer.clone());
        let dispatch = tracing::Dispatch::new(subscriber(output, LogFormat::Json, true));
        std::thread::scope(|scope| {
            for worker in 0..4 {
                let dispatch = &dispatch;
                scope.spawn(move || {
                    tracing::dispatcher::with_default(dispatch, || {
                        for index in 0..25 {
                            tracing::debug!(worker, index, "record");
                        }
                    })
                });
            }
        });
        let text = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        assert_eq!(text.lines().count(), 100);
        for line in text.lines() {
            let record: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(record["msg"], "record");
        }
    }

    #[test]
    fn write_and_flush_errors_are_retained() {
        struct Broken(bool);
        impl Write for Broken {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.0 {
                    Err(std::io::Error::other("write failed"))
                } else {
                    Ok(bytes.len())
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("flush failed"))
            }
        }
        for (write_fails, expected) in [(true, "write failed"), (false, "flush failed")] {
            let output = LogHandle::new(Broken(write_fails));
            tracing::subscriber::with_default(
                subscriber(output.clone(), LogFormat::Json, false),
                || {
                    tracing::error!("diagnostic");
                },
            );
            assert_eq!(output.failure().as_deref(), Some(expected));
        }
    }
}
