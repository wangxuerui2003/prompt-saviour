use std::ptr;

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedUIElementAttribute, kAXValueAttribute, AXUIElementCopyAttributeValue,
    AXUIElementCreateSystemWide, AXUIElementRef,
};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::string::CFString;
use tracing::debug;

pub fn read_focused_text() -> anyhow::Result<Option<String>> {
    unsafe {
        if !accessibility_sys::AXIsProcessTrusted() {
            anyhow::bail!("accessibility permission not granted");
        }

        let system = AXUIElementCreateSystemWide();
        if system.is_null() {
            anyhow::bail!("AXUIElementCreateSystemWide returned null");
        }

        let attr = CFString::new(kAXFocusedUIElementAttribute);
        let mut focused_ref: CFTypeRef = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(
            system,
            attr.as_concrete_TypeRef() as _,
            &mut focused_ref,
        );
        if err != kAXErrorSuccess || focused_ref.is_null() {
            return Ok(None);
        }

        let focused = focused_ref as AXUIElementRef;
        read_element_value(focused)
    }
}

unsafe fn read_element_value(element: AXUIElementRef) -> anyhow::Result<Option<String>> {
    let attr = CFString::new(kAXValueAttribute);
    let mut value_ref: CFTypeRef = ptr::null_mut();
    let err = AXUIElementCopyAttributeValue(
        element,
        attr.as_concrete_TypeRef() as _,
        &mut value_ref,
    );
    if err != kAXErrorSuccess || value_ref.is_null() {
        return Ok(None);
    }

    let cf_type = CFType::wrap_under_create_rule(value_ref as *mut _);
    if let Some(cf_string) = cf_type.downcast::<CFString>() {
        let text = cf_string.to_string();
        if text.trim().is_empty() {
            return Ok(None);
        }
        debug!(chars = text.chars().count(), "ax captured focused text");
        return Ok(Some(text));
    }

    Ok(None)
}
