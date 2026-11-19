//! Run devloop at login with launchd (docs/DEVELOPMENT.md §5).

use anyhow::Context;
use std::path::{Path, PathBuf};

pub const LABEL: &str = "dev.mnemo.devloop";

pub fn plist_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join("Library/LaunchAgents").join(format!("{LABEL}.plist")))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The launchd property list. PATH includes the usual install folders so `claude`, `git`,
/// `cargo` and `just` are found.
pub fn plist(exe: &Path, repo: &Path, home: &str) -> String {
    let path = format!(
        "{home}/.local/bin:{home}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
    );
    let log = repo.join(".devloop/devloop.log");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>--repo</string>
    <string>{repo}</string>
    <string>run</string>
  </array>
  <key>WorkingDirectory</key><string>{repo}</string>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>ThrottleInterval</key><integer>60</integer>
  <key>EnvironmentVariables</key>
  <dict><key>PATH</key><string>{path}</string></dict>
  <key>StandardOutPath</key><string>{log}</string>
  <key>StandardErrorPath</key><string>{log}</string>
</dict>
</plist>
"#,
        exe = xml_escape(&exe.display().to_string()),
        repo = xml_escape(&repo.display().to_string()),
        path = xml_escape(&path),
        log = xml_escape(&log.display().to_string()),
    )
}

fn uid() -> anyhow::Result<String> {
    let out = std::process::Command::new("id").arg("-u").output()?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn install(repo: &Path) -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?.canonicalize()?;
    let home = std::env::var("HOME")?;
    let path = plist_path()?;
    std::fs::create_dir_all(path.parent().expect("plist has a parent"))?;
    std::fs::create_dir_all(repo.join(".devloop"))?;
    std::fs::write(&path, plist(&exe, repo, &home))?;
    let domain = format!("gui/{}", uid()?);
    let _ = std::process::Command::new("launchctl").args(["bootout", &format!("{domain}/{LABEL}")]).output();
    let status = std::process::Command::new("launchctl").args(["bootstrap", &domain]).arg(&path).status()?;
    anyhow::ensure!(status.success(), "launchctl bootstrap failed");
    Ok(path)
}

pub fn uninstall() -> anyhow::Result<()> {
    let domain = format!("gui/{}", uid()?);
    let _ = std::process::Command::new("launchctl").args(["bootout", &format!("{domain}/{LABEL}")]).status();
    let path = plist_path()?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_runs_devloop_for_the_repo() {
        let text = plist(Path::new("/x/devloop"), Path::new("/p/langtype"), "/Users/me");
        assert!(text.contains("<string>/x/devloop</string>"));
        assert!(text.contains("<string>run</string>"));
        assert!(text.contains("/p/langtype/.devloop/devloop.log"));
        assert!(text.contains("/Users/me/.local/bin"));
    }
}
