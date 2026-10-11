//! `taskr`: a to-do list for the terminal.

mod task;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::task::{Priority, Task};

/// A to-do list for the terminal. Tasks are stored in a JSON file.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// The tasks file
    #[arg(
        long,
        short,
        global = true,
        default_value = "tasks.json",
        env = "TASKR_FILE"
    )]
    file: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add a task
    Add {
        /// What to do
        #[arg(value_parser = clap::builder::NonEmptyStringValueParser::new())]
        title: String,
        /// How urgent it is
        #[arg(long, short, value_enum, default_value_t = Priority::Normal)]
        priority: Priority,
    },
    /// List the tasks that aren't done yet
    List {
        /// Include the tasks that are done
        #[arg(long, short)]
        all: bool,
        /// Print JSON instead of a table, for scripts
        #[arg(long)]
        json: bool,
    },
    /// Mark a task as done
    Done {
        /// The task's id, as shown by `taskr list`
        id: u32,
    },
    /// Delete a task
    Remove {
        /// The task's id, as shown by `taskr list`
        id: u32,
    },
    /// Delete every task that is done
    Clear,
}

fn main() -> ExitCode {
    // On --help, --version or a usage error, parse() prints the message and
    // exits by itself (code 0 for help and version, 2 for usage errors).
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // {:#} prints the error with its whole chain of context on one line.
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let mut tasks = task::load(&cli.file)?;
    match cli.command {
        Command::Add { title, priority } => {
            let id = tasks
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .context("no task ids left")?;
            tasks.push(Task {
                id,
                title: title.clone(),
                priority,
                done: false,
            });
            task::save(&cli.file, &tasks)?;
            println!("Added task {id}: {title}");
        }
        Command::List { all, json } => {
            let shown: Vec<&Task> = tasks.iter().filter(|t| all || !t.done).collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&shown)?);
            } else if shown.is_empty() {
                println!("No tasks.");
            } else {
                for task in shown {
                    println!("{task}");
                }
            }
        }
        Command::Done { id } => {
            let task = tasks
                .iter_mut()
                .find(|t| t.id == id)
                .with_context(|| format!("no task with id {id}"))?;
            task.done = true;
            let title = task.title.clone();
            task::save(&cli.file, &tasks)?;
            println!("Completed task {id}: {title}");
        }
        Command::Remove { id } => {
            let index = tasks
                .iter()
                .position(|t| t.id == id)
                .with_context(|| format!("no task with id {id}"))?;
            let removed = tasks.remove(index);
            task::save(&cli.file, &tasks)?;
            println!("Removed task {id}: {}", removed.title);
        }
        Command::Clear => {
            let before = tasks.len();
            tasks.retain(|t| !t.done);
            let removed = before - tasks.len();
            task::save(&cli.file, &tasks)?;
            println!("Removed {removed} done tasks");
        }
    }
    Ok(())
}
