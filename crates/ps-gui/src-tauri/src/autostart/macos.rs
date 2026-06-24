use std::path::PathBuf;
use std::process::Command;

const LABEL: &str = "com.promptsaviour.app";

fn plist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist"))
}

fn plist_body(exe: &PathBuf) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
</dict>
</plist>
"#,
        exe.display()
    )
}

pub fn is_enabled() -> bool {
    plist_path().exists()
}

pub fn set_enabled(enabled: bool) -> anyhow::Result<bool> {
    let path = plist_path();
    if enabled {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let exe = std::env::current_exe()?;
        std::fs::write(&path, plist_body(&exe))?;
        let _ = Command::new("launchctl")
            .args(["bootout", "gui/501"])
            .arg(&path)
            .status();
        let _ = Command::new("launchctl")
            .args(["bootstrap", "gui/501"])
            .arg(&path)
            .status();
    } else if path.exists() {
        let _ = Command::new("launchctl")
            .args(["bootout", "gui/501"])
            .arg(&path)
            .status();
        let _ = std::fs::remove_file(path);
    }
    Ok(enabled)
}
