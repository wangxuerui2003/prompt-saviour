use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct FileLogGuard {
    _writer: Arc<Mutex<std::fs::File>>,
}

struct FileWriter(Arc<Mutex<std::fs::File>>);

impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

pub fn log_file_path() -> anyhow::Result<PathBuf> {
    Ok(ps_core::data_dir()?.join("prompt-saviour.log"))
}

pub fn init_file_logging() -> anyhow::Result<FileLogGuard> {
    let path = log_file_path()?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let writer = Arc::new(Mutex::new(file));
    let file_writer = FileWriter(Arc::clone(&writer));

    let _ = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("prompt_saviour=info".parse()?))
        .with(fmt::layer().with_writer(Mutex::new(file_writer)))
        .try_init();

    Ok(FileLogGuard {
        _writer: writer,
    })
}

pub fn read_recent_logs(lines: usize) -> anyhow::Result<String> {
    let path = log_file_path()?;
    if !path.exists() {
        return Ok(String::new());
    }
    let content = fs::read_to_string(path)?;
    if content.is_empty() {
        return Ok(String::new());
    }
    let tail: Vec<&str> = content.lines().rev().take(lines).collect();
    Ok(tail.into_iter().rev().collect::<Vec<_>>().join("\n"))
}
