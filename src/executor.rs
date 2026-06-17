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
