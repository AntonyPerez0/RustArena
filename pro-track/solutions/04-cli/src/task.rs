//! Tasks, and the JSON file that stores them.

use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use anyhow::{Context, Result};
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
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err).with_context(|| format!("could not read {}", path.display()));
        }
    };
    serde_json::from_str(&text).with_context(|| format!("could not parse {}", path.display()))
}

/// Writes `tasks` to `path` as pretty-printed JSON. The data goes to a
/// temporary file first, which is then renamed over `path`, so a crash can
/// never leave a half-written tasks file behind.
pub fn save(path: &Path, tasks: &[Task]) -> Result<()> {
    let mut json = serde_json::to_string_pretty(tasks).context("could not serialize the tasks")?;
    json.push('\n');
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, json).with_context(|| format!("could not write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("could not write {}", path.display()))
}
