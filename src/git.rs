use std::process::Command;

pub fn get_recent_changes() -> Option<String> {
    // 1. Check if git is available and we are in a git repository
    let is_git_repo = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_git_repo {
        return None;
    }

    // 2. Try to get uncommitted changes first (working directory diff)
    let mut diff = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    // 3. If no uncommitted changes, get the last commit
    if diff.trim().is_empty() {
        diff = Command::new("git")
            .arg("show")
            .arg("HEAD")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
    }

    // Limit diff size to prevent token blowup (e.g., limit to ~5,000 chars)
    if diff.len() > 5000 {
        diff.truncate(5000);
        diff.push_str("\n... (diff truncated)");
    }

    if diff.trim().is_empty() {
        None
    } else {
        Some(diff)
    }
}
