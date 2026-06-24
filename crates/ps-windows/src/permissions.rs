use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub ui_automation: bool,
    pub input_monitoring: bool,
}

pub fn check_permissions() -> PermissionStatus {
    let uia_ok = uia_smoke_test();
    PermissionStatus {
        ui_automation: uia_ok,
        input_monitoring: ps_input::probe_input_monitoring(),
    }
}

pub fn prompt_for_permissions() -> PermissionStatus {
    check_permissions()
}

fn uia_smoke_test() -> bool {
    super::uia::read_focused_text().is_ok()
}
