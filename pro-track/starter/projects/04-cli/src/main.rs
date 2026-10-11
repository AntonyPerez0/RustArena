//! `taskr`: a to-do list for the terminal.

mod task;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::task::{Priority, Task};

/// A to-do list for the terminal. Tasks are stored in a JSON file.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// The tasks file
    // TODO 5: also read it from the TASKR_FILE environment variable.
    #[arg(long, short, global = true, default_value = "tasks.json")]
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
    // TODO 3: add the Done, Remove and Clear subcommands (see the README).
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
        // TODO 4: handle Done, Remove and Clear too.
        Command::Add { title, priority } => {
            let id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
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
    }
    Ok(())
}
