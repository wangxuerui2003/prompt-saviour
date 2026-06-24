use uiautomation::UIAutomation;

pub fn read_focused_text() -> anyhow::Result<Option<String>> {
    let automation = UIAutomation::new()?;
    let focused = automation.get_focused_element()?;
    if focused.is_null() {
        return Ok(None);
    }
    let patterns = focused.get_supported_patterns();
    if patterns.contains(&uiautomation::types::PatternId::Value) {
        if let Ok(value) = focused.get_value_pattern() {
            let text = value.get_value()?;
            if !text.trim().is_empty() {
                return Ok(Some(text));
            }
        }
    }
    if patterns.contains(&uiautomation::types::PatternId::Text) {
        if let Ok(text_pattern) = focused.get_text_pattern() {
            let range = text_pattern.get_document_range()?;
            let text = range.get_text(-1)?;
            if !text.trim().is_empty() {
                return Ok(Some(text));
            }
        }
    }
    let name = focused.get_name()?;
    if !name.trim().is_empty() && name.chars().count() >= 8 {
        return Ok(Some(name));
    }
    Ok(None)
}
