use std::path::PathBuf;
use std::process::Command;

const APP_NAME: &str = "PromptSaviour";

pub fn is_enabled() -> bool {
    read_run_key().is_ok()
}

pub fn set_enabled(enabled: bool) -> anyhow::Result<bool> {
    if enabled {
        let exe = std::env::current_exe()?;
        write_run_key(&exe)?;
    } else {
        remove_run_key()?;
    }
    Ok(enabled)
}

fn write_run_key(exe: &PathBuf) -> anyhow::Result<()> {
    use std::os::windows::process::CommandExt;
    let path = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let status = Command::new("reg")
        .args([
            "add",
            path,
            "/v",
            APP_NAME,
            "/t",
            "REG_SZ",
            "/d",
            &exe.display().to_string(),
            "/f",
        ])
        .creation_flags(0x08000000)
        .status()?;
    anyhow::ensure!(status.success(), "failed to write registry Run key");
    Ok(())
}

fn remove_run_key() -> anyhow::Result<()> {
    use std::os::windows::process::CommandExt;
    let path = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let _ = Command::new("reg")
        .args(["delete", path, "/v", APP_NAME, "/f"])
        .creation_flags(0x08000000)
        .status();
    Ok(())
}

fn read_run_key() -> anyhow::Result<String> {
    use std::os::windows::process::CommandExt;
    let output = Command::new("reg")
        .args([
            "query",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            APP_NAME,
        ])
        .creation_flags(0x08000000)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("run key missing");
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
