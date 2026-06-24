use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub at_spi: bool,
    pub input_monitoring: bool,
}

pub fn check_permissions() -> PermissionStatus {
    let at_spi = crate::atspi_text::probe_atspi().unwrap_or(false);
    PermissionStatus {
        at_spi,
        input_monitoring: ps_input::probe_input_monitoring(),
    }
}

pub fn prompt_for_permissions() -> PermissionStatus {
    check_permissions()
}
