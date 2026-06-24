use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("tokio runtime for AT-SPI"))
}

pub fn probe_atspi() -> anyhow::Result<bool> {
    use atspi::connection::AccessibilityConnection;

    runtime()
        .block_on(async { AccessibilityConnection::new().await })
        .map(|_| true)
        .map_err(Into::into)
}

/// GUI text capture via AT-SPI Text interface.
/// Keystroke capture remains the primary Linux fallback until this path is expanded.
pub fn read_focused_text() -> anyhow::Result<Option<String>> {
    let _ = probe_atspi()?;
    Ok(None)
}
