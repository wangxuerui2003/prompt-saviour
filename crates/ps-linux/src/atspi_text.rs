use atspi::connection::AccessibilityConnection;
use atspi::proxy::accessible::AccessibleProxy;

pub fn probe_atspi() -> anyhow::Result<bool> {
    let conn = AccessibilityConnection::new()?;
    let _ = conn.connection();
    Ok(true)
}

pub fn read_focused_text() -> anyhow::Result<Option<String>> {
    let conn = AccessibilityConnection::new()?;
    let conn_ref = conn.connection();
    let proxy = AccessibleProxy::new(conn_ref).map_err(|e| anyhow::anyhow!(e))?;
    let focused = proxy.get_focused().map_err(|e| anyhow::anyhow!(e))?;
    let text = focused
        .get_text(0, i32::MAX)
        .map_err(|e| anyhow::anyhow!(e))?;
    if text.trim().chars().count() >= 8 {
        Ok(Some(text))
    } else {
        Ok(None)
    }
}
