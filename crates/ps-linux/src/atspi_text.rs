use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("tokio runtime for AT-SPI"))
}

pub fn probe_atspi() -> anyhow::Result<bool> {
    use atspi::connection::AccessibilityConnection;

    match runtime().block_on(async { AccessibilityConnection::new().await }) {
        Ok(_) => Ok(true),
        Err(err) => Err(err.into()),
    }
}

/// GUI text capture via AT-SPI.
/// Keystroke capture remains the primary Linux fallback until Text interface wiring lands.
pub fn read_focused_text() -> anyhow::Result<Option<String>> {
    let _available = probe_atspi()?;
    Ok(None)
}
