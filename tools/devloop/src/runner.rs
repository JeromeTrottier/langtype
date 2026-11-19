//! Running one agent: git worktree + headless Claude Code (docs/DEVELOPMENT.md §3).

use crate::config::Config;
use crate::outcome::{self, Outcome};
use crate::schedule::Action;
use crate::state;
use crate::task::Task;
use anyhow::{Context, bail};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Run a git command in `dir`, returning stdout.
pub fn git(dir: &Path, args: &[&str]) -> anyhow::Result<String> {
    let out =
        std::process::Command::new("git").arg("-C").arg(dir).args(args).output().context("running git")?;
    if !out.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn worktree_path(repo: &Path, config: &Config, task: &Task) -> PathBuf {
    repo.join(&config.worktrees_dir).join(format!("{:04}", task.meta.id))
}

/// Make sure the task has a worktree on its branch; returns its path. Records the branch on the task.
pub fn ensure_worktree(repo: &Path, config: &Config, task: &mut Task) -> anyhow::Result<PathBuf> {
    let path = worktree_path(repo, config, task);
    let branch = task.meta.branch.clone().unwrap_or_else(|| task.branch_name());
    if path.join(".git").exists() {
        task.meta.branch = Some(branch);
        return Ok(path);
    }
    let path_str = path.to_string_lossy().to_string();
    let branch_exists =
        git(repo, &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{branch}")]).is_ok();
    if branch_exists {
        git(repo, &["worktree", "add", &path_str, &branch])?;
    } else {
        git(repo, &["worktree", "add", "-b", &branch, &path_str, &config.base_branch])?;
    }
    task.meta.branch = Some(branch);
    Ok(path)
}

fn prompt(action: &Action, task_path: Option<&Path>) -> String {
    let task = task_path.map(|p| p.display().to_string()).unwrap_or_default();
    match action {
        Action::Implement(_) => format!("/implement-task {task}"),
        Action::Fix(_) => format!("/fix-task {task}"),
        Action::Review(_) => format!("/review-task {task}"),
        Action::Plan(milestone) => format!("/plan-milestone {milestone}"),
    }
}

/// Run the agent for `action` in `cwd`. Writes the raw output to `.devloop/runs/…` and returns the outcome.
pub async fn run_agent(
    repo: &Path,
    config: &Config,
    action: &Action,
    cwd: &Path,
    task_path: Option<&Path>,
) -> (Outcome, PathBuf) {
    let target = action.task_id().map(|id| format!("{id:04}")).unwrap_or_else(|| "planner".into());
    let log_path = state::run_log_path(repo, &target, action.role());
    let tasks_dir = crate::task::tasks_dir(repo);

    let mut cmd = Command::new(&config.claude_bin);
    cmd.current_dir(cwd)
        .arg("-p")
        .arg(prompt(action, task_path))
        .args(["--output-format", "stream-json", "--verbose", "--permission-mode", "acceptEdits"])
        .arg("--max-turns")
        .arg(config.max_turns_per_run.to_string())
        .arg("--add-dir")
        .arg(&tasks_dir)
        .env("DEVLOOP", "1")
        .env("DEVLOOP_ROLE", action.role())
        .env("DEVLOOP_TARGET", &target)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Own process group, so stopping the agent also stops the tools it started.
        .process_group(0)
        .kill_on_drop(true);

    let outcome = match cmd.spawn() {
        Err(e) => Outcome::Failed {
            message: format!("could not start `{}`: {e}", config.claude_bin),
            usage: Default::default(),
        },
        Ok(child) => {
            let pid = child.id();
            let limit = Duration::from_secs(config.run_timeout_minutes * 60);
            let result = tokio::time::timeout(limit, watch(repo, config, child)).await;
            if result.is_err() {
                stop_group(pid);
            }
            match result {
                Err(_) => Outcome::Failed {
                    message: format!("timed out after {} minutes", config.run_timeout_minutes),
                    usage: Default::default(),
                },
                Ok(Err(e)) => {
                    Outcome::Failed { message: format!("waiting for agent: {e}"), usage: Default::default() }
                }
                Ok(Ok(run)) => {
                    let record = serde_json::json!({
                        "role": action.role(), "cwd": cwd, "exit": run.exit,
                        "stopped_for_cap": run.capped.is_some(), "stdout": run.stdout, "stderr": run.stderr,
                    });
                    let _ = state::write_json(&log_path, &record);
                    match run.capped {
                        Some(outcome) => outcome,
                        None => outcome::interpret(&run.stdout, &run.stderr, Utc::now()),
                    }
                }
            }
        }
    };
    (outcome, log_path)
}

struct Finished {
    stdout: String,
    stderr: String,
    exit: Option<i32>,
    /// Set when devloop stopped the agent because a usage window crossed the cap.
    capped: Option<Outcome>,
}

/// Read the agent's event stream as it runs. Every `rate_limit_event` updates
/// `.devloop/limits.json`; if a window reaches `max_usage_percent`, the agent is stopped.
async fn watch(repo: &Path, config: &Config, mut child: tokio::process::Child) -> std::io::Result<Finished> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

    let stdout = child.stdout.take().expect("stdout is piped");
    let mut stderr = child.stderr.take().expect("stderr is piped");
    let stderr_task = tokio::spawn(async move {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text).await;
        text
    });

    let mut lines = BufReader::new(stdout).lines();
    let mut out = String::new();
    let mut capped = None;
    while let Some(line) = lines.next_line().await? {
        if let Some(windows) = outcome::rate_windows(&line) {
            let _ = state::save_limits(repo, &windows);
            if let Some(w) = outcome::over_cap(&windows, config.max_usage_percent, Utc::now()) {
                let reason = if w.status == "rejected" {
                    format!("Claude usage limit reached ({} window)", w.name)
                } else {
                    format!(
                        "{} usage at {:.0}% (cap {:.0}%)",
                        w.name.replace('_', "-"),
                        w.utilization * 100.0,
                        config.max_usage_percent
                    )
                };
                capped = Some(Outcome::UsageLimit { resume_at: w.resets_at, reason });
                out.push_str(&line);
                out.push('\n');
                stop_group(child.id());
                let _ = child.kill().await;
                break;
            }
        }
        out.push_str(&line);
        out.push('\n');
    }
    let status = child.wait().await?;
    // A stopped agent's leftover children may hold stderr open; don't wait on them for long.
    let stderr = tokio::time::timeout(Duration::from_secs(5), stderr_task)
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or_default();
    Ok(Finished { stdout: out, stderr, exit: status.code(), capped })
}

/// Stop the agent's whole process group (it was started with `process_group(0)`, so its
/// group id is its pid): SIGTERM first, SIGKILL shortly after.
fn stop_group(pid: Option<u32>) {
    let Some(pid) = pid else { return };
    let group = format!("-{pid}");
    let _ = std::process::Command::new("kill").args(["-TERM", "--", &group]).stderr(Stdio::null()).status();
    std::thread::sleep(Duration::from_millis(500));
    let _ = std::process::Command::new("kill").args(["-KILL", "--", &group]).stderr(Stdio::null()).status();
}

/// macOS notification (skipped when `DEVLOOP_QUIET=1`); failures are ignored.
pub fn notify(message: &str) {
    if std::env::var("DEVLOOP_QUIET").as_deref() != Ok("1") {
        let script = format!("display notification {:?} with title \"devloop\"", message);
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    println!("{} notify: {message}", chrono::Local::now().format("%H:%M:%S"));
}
