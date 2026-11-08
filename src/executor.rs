use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Executor;

impl Executor {
    pub fn run_command(command: &str) -> Result<String, String> {
        let os = std::env::consts::OS;
        
        let output = if os == "windows" {
            Command::new("powershell")
                .args(["-Command", command])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", command])
                .output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                
                let result = format!(
                    "Exit code: {}\nStdout:\n{}\nStderr:\n{}",
                    out.status.code().unwrap_or(-1),
                    stdout,
                    stderr
                );
                
                if out.status.success() {
                    Ok(result)
                } else {
                    Err(result)
                }
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }

    pub fn write_file(file_path: &str, content: &str) -> Result<String, String> {
        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create directories: {}", e));
                }
            }
        }
        
        match fs::write(path, content) {
            Ok(_) => Ok(format!("Successfully wrote file: {}", file_path)),
            Err(e) => Err(format!("Failed to write file: {}", e)),
        }
    }

    pub fn read_file(file_path: &str) -> Result<String, String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }
        
        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_success_contains_stdout() {
        let result = Executor::run_command("echo lightagent-test").unwrap();
        assert!(result.contains("lightagent-test"));
        assert!(result.contains("Exit code: 0"));
    }

    #[test]
    fn test_run_command_failure_returns_error() {
        let result = Executor::run_command("exit 3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Exit code: 3"));
    }

    #[test]
    fn test_write_then_read_file() {
        let directory = std::env::temp_dir().join(format!("lightagent-test-{}", std::process::id()));
        let file_path = directory.join("nested").join("note.txt");
        let file_path_text = file_path.to_string_lossy().to_string();

        Executor::write_file(&file_path_text, "content").unwrap();
        assert_eq!(Executor::read_file(&file_path_text).unwrap(), "content");

        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn test_read_missing_file_returns_error() {
        assert!(Executor::read_file("/nonexistent/lightagent/file.txt").is_err());
    }
}
