use accessibility_sys::AXIsProcessTrustedWithOptions;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation::base::TCFType;
use tracing::info;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub accessibility: bool,
    pub input_monitoring: bool,
}

pub fn check_permissions() -> PermissionStatus {
    let trusted = is_accessibility_trusted();
    PermissionStatus {
        accessibility: trusted,
        input_monitoring: trusted,
    }
}

pub fn prompt_for_permissions() -> PermissionStatus {
    prompt_accessibility();
    check_permissions()
}

fn is_accessibility_trusted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}

fn prompt_accessibility() {
    unsafe {
        let key = CFString::new("AXPrompt");
        let value = CFBoolean::true_value();
        let dict = CFDictionary::from_CFType_pairs(&[(
            CFString::wrap_under_get_rule(key.as_concrete_TypeRef()),
            value.clone(),
        )]);
        let _ = AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as _);
    }
    info!("accessibility permission prompt displayed");
}
