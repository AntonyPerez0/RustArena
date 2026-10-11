//! Tasks, and the JSON file that stores them.

use std::fmt;
use std::path::Path;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// How urgent a task is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // pad (not write_str) so that widths like {:<6} work.
        f.pad(match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
        })
    }
}

/// One to-do item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub priority: Priority,
    pub done: bool,
}

/// One line of `taskr list`, like `  3 [x] high   Buy milk`.
impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mark = if self.done { 'x' } else { ' ' };
        write!(
            f,
            "{:>3} [{mark}] {:<6} {}",
            self.id, self.priority, self.title
        )
    }
}

/// Reads the tasks stored in `path`. A file that doesn't exist yet simply
/// holds no tasks; any other problem is an error that names the file.
pub fn load(path: &Path) -> Result<Vec<Task>> {
    // TODO 1: read the file with std::fs::read_to_string. If the error's kind()
    // is std::io::ErrorKind::NotFound, return an empty Vec. Otherwise add
    // context: "could not read <path>". Then parse it with serde_json::from_str,
    // with the context "could not parse <path>".
    todo!("load the tasks from {}", path.display())
}

/// Writes `tasks` to `path` as pretty-printed JSON. The data goes to a
/// temporary file first, which is then renamed over `path`, so a crash can
/// never leave a half-written tasks file behind.
pub fn save(path: &Path, tasks: &[Task]) -> Result<()> {
    // TODO 2: serde_json::to_string_pretty, then std::fs::write to a temporary
    // file next to `path` (its name plus ".tmp") and std::fs::rename it to
    // `path`. Errors get the context "could not write <file>".
    todo!("save {} tasks to {}", tasks.len(), path.display())
}
