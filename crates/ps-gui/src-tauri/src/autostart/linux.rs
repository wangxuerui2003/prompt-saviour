use std::path::PathBuf;

const DESKTOP_FILE: &str = "prompt-saviour.desktop";

fn desktop_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/autostart")
        .join(DESKTOP_FILE)
}

fn desktop_body(exe: &PathBuf) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Prompt Saviour\nExec={}\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    )
}

pub fn is_enabled() -> bool {
    desktop_path().exists()
}

pub fn set_enabled(enabled: bool) -> anyhow::Result<bool> {
    let path = desktop_path();
    if enabled {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let exe = std::env::current_exe()?;
        std::fs::write(&path, desktop_body(&exe))?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(enabled)
}
