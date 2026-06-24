use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use crate::CaptureDaemon;

pub fn run_daemon() -> Result<()> {
    let daemon = Arc::new(CaptureDaemon::new());
    daemon.start()?;

    let stop_daemon = Arc::clone(&daemon);
    ctrlc::set_handler(move || {
        stop_daemon.stop();
    })?;

    let status = daemon.status()?;
    println!("prompt-saviour running (db: {})", status.db_path);
    println!("Use `prompt-saviour list` / `prompt-saviour recover` in another terminal.");

    while daemon.is_running() {
        std::thread::sleep(Duration::from_millis(200));
    }

    println!("prompt-saviour stopped.");
    Ok(())
}
