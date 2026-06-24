use uiautomation::UIAutomation;

pub fn read_focused_text() -> anyhow::Result<Option<String>> {
    let automation = UIAutomation::new()?;
    let element = automation.get_focused_element()?;
    let name = element.get_name().unwrap_or_default();
    if name.trim().chars().count() >= 8 {
        Ok(Some(name))
    } else {
        Ok(None)
    }
}
