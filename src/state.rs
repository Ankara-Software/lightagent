use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub timestamp: String,
    pub step_type: String, // "task_start" | "command_run" | "file_edit" | "completed" | "error"
    pub payload: String,
}

pub struct StateManager {
    dir_path: String,
    file_path: String,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            dir_path: ".agent".to_string(),
            file_path: ".agent/state.jsonl".to_string(),
        }
    }

    pub fn is_state_present(&self) -> bool {
        Path::new(&self.file_path).exists()
    }

    pub fn append_step(&self, step_type: &str, payload: &str) -> io::Result<()> {
        if !Path::new(&self.dir_path).exists() {
            fs::create_dir_all(&self.dir_path)?;
        }

        let step = Step {
            timestamp: Utc::now().to_rfc3339(),
            step_type: step_type.to_string(),
            payload: payload.to_string(),
        };

        let serialized = serde_json::to_string(&step)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", serialized)?;
        Ok(())
    }

    pub fn load_steps(&self) -> io::Result<Vec<Step>> {
        if !self.is_state_present() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut steps = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }
            if let Ok(step) = serde_json::from_str::<Step>(&line_content) {
                steps.push(step);
            }
        }
        Ok(steps)
    }

    pub fn clear_state(&self) -> io::Result<()> {
        if self.is_state_present() {
            fs::remove_file(&self.file_path)?;
        }
        if Path::new(&self.dir_path).exists() {
            fs::remove_dir(&self.dir_path)?;
        }
        Ok(())
    }
}
