use core_foundation::array::CFArray;
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::window::{
    kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo,
};
use objc2_app_kit::NSRunningApplication;
use ps_core::SessionContext;

pub fn frontmost_session() -> Option<SessionContext> {
    let (pid, window_title, owner_name) = frontmost_window_info()?;
    let bundle_id = bundle_id_for_pid(pid).unwrap_or_else(|| format!("pid:{pid}"));
    let app_name = owner_name.unwrap_or_else(|| bundle_id.clone());
    let window_title = window_title.unwrap_or_else(|| app_name.clone());

    Some(SessionContext {
        bundle_id,
        app_name,
        pid,
        window_title,
    })
}

fn frontmost_window_info() -> Option<(u32, Option<String>, Option<String>)> {
    unsafe {
        let window_list = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID);
        if window_list.is_null() {
            return None;
        }
        let array: CFArray<CFDictionary<CFString, CFType>> =
            CFArray::wrap_under_create_rule(window_list as _);

        for item in array.iter() {
            let dict =
                CFDictionary::<CFString, CFType>::wrap_under_get_rule(item.as_concrete_TypeRef());
            let dict_ref = dict.as_concrete_TypeRef() as CFTypeRef;
            let layer = get_i64(dict_ref, "kCGWindowLayer")?;
            if layer != 0 {
                continue;
            }
            let pid = get_i64(dict_ref, "kCGWindowOwnerPID")? as u32;
            let owner = get_string(dict_ref, "kCGWindowOwnerName");
            let title = get_string(dict_ref, "kCGWindowName").filter(|t| !t.is_empty());
            return Some((pid, title, owner));
        }
        None
    }
}

unsafe fn get_i64(dict: CFTypeRef, key: &str) -> Option<i64> {
    let dict = CFDictionary::<CFString, CFType>::wrap_under_get_rule(dict as _);
    let key = CFString::new(key);
    let value = dict.find(key.as_concrete_TypeRef())?;
    value.downcast::<CFNumber>()?.to_i64()
}

unsafe fn get_string(dict: CFTypeRef, key: &str) -> Option<String> {
    let dict = CFDictionary::<CFString, CFType>::wrap_under_get_rule(dict as _);
    let key = CFString::new(key);
    let value = dict.find(key.as_concrete_TypeRef())?;
    value.downcast::<CFString>().map(|s| s.to_string())
}

fn bundle_id_for_pid(pid: u32) -> Option<String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid as i32)?;
    app.bundleIdentifier().map(|b| b.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmost_session_has_app_name() {
        let session = frontmost_session();
        if let Some(s) = session {
            assert!(!s.app_name.is_empty());
            assert!(s.pid > 0);
        }
    }
}
