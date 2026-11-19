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
        .args(["--output-format", "json", "--permission-mode", "acceptEdits"])
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
        .kill_on_drop(true);

    let outcome = match cmd.spawn() {
        Err(e) => Outcome::Failed {
            message: format!("could not start `{}`: {e}", config.claude_bin),
            usage: Default::default(),
        },
        Ok(child) => {
            let limit = Duration::from_secs(config.run_timeout_minutes * 60);
            match tokio::time::timeout(limit, child.wait_with_output()).await {
                Err(_) => Outcome::Failed {
                    message: format!("timed out after {} minutes", config.run_timeout_minutes),
                    usage: Default::default(),
                },
                Ok(Err(e)) => {
                    Outcome::Failed { message: format!("waiting for agent: {e}"), usage: Default::default() }
                }
                Ok(Ok(out)) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    let record = serde_json::json!({ "role": action.role(), "cwd": cwd, "exit": out.status.code(), "stdout": stdout, "stderr": stderr });
                    let _ = state::write_json(&log_path, &record);
                    outcome::interpret(&stdout, &stderr, Utc::now())
                }
            }
        }
    };
    (outcome, log_path)
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
