use ps_core::SessionContext;

pub fn frontmost_session() -> Option<SessionContext> {
    let window = active_win_pos_rs::get_active_window().ok()??;
    let bundle_id = window
        .process_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| window.app_name.clone());

    Some(SessionContext {
        bundle_id,
        app_name: window.app_name,
        pid: window.process_id as u32,
        window_title: window.title,
    })
}
