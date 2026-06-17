use std::process::Command;

pub struct Git;

impl Git {
    pub fn is_repo() -> bool {
        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output();
        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    pub fn get_commit_hash() -> String {
        let output = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output();
        match output {
            Ok(out) => {
                if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                } else {
                    "unknown".to_string()
                }
            }
            Err(_) => "unknown".to_string(),
        }
    }

    pub fn get_branch() -> String {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output();
        match output {
            Ok(out) => {
                if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                } else {
                    "unknown".to_string()
                }
            }
            Err(_) => "unknown".to_string(),
        }
    }

    pub fn get_status() -> String {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output();
        match output {
            Ok(out) => {
                if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        }
    }
}
