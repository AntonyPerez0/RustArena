//! `wordfreq`: prints the most frequent words in the text on standard input.

use std::io::{self, Read};
use std::process::ExitCode;

const USAGE: &str = "\
usage: wordfreq [--top N] < FILE

Prints the most frequent words in the text read from standard input.

options:
  -n, --top N     how many words to print (default: 10)
  -h, --help      print this help
  -V, --version   print the version";

/// What the command line asked for.
#[derive(Debug, PartialEq)]
enum Command {
    Run { top: usize },
    Help,
    Version,
}

/// Parses the arguments (without the program name).
fn parse_args(args: &Vec<String>) -> Result<Command, String> {
    let mut top = 10;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "-n" | "--top" => {
                i = i + 1;
                let Some(value) = args.get(i) else {
                    return Err(format!("{arg} needs a value"));
                };
                top = match value.parse::<usize>() {
                    Ok(n) if n > 0 => n,
                    _ => return Err(format!("{arg} needs a whole number above 0, not '{value}'")),
                };
            }
            _ => return Err(format!("unexpected argument '{arg}'")),
        }
        i = i + 1;
    }
    return Ok(Command::Run { top: top });
}

fn main()->ExitCode {
    let args:Vec<String> = std::env::args().skip(1).collect();
    let top = match parse_args(&args) {
        Ok(Command::Run { top }) => top,
        Ok(Command::Help) => { println!("{USAGE}"); return ExitCode::SUCCESS; }
        Ok(Command::Version) => {
            // Cargo sets CARGO_PKG_VERSION from Cargo.toml when it compiles the crate.
            println!("wordfreq {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            // Usage errors go to standard error, with exit code 2 (the Unix convention).
            eprintln!("wordfreq: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    let mut text = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut text) {
        eprintln!("wordfreq: cannot read standard input: {err}");
        return ExitCode::FAILURE;
    }

    let counts = wordfreq::count_words(&text);
    for (word, count) in wordfreq::top_words(&counts, top) { println!("{word} {count}"); }
    ExitCode::SUCCESS
}
