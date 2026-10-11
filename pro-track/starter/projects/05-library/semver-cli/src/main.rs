//! `semver`: a small command-line front end for the semver-lite library.
//!
//! The binary only reads arguments, calls the library and prints results. All
//! the logic lives in the library, where other programs can reuse it and the
//! tests can reach it without starting a process.

use std::error::Error;
use std::process::ExitCode;

use semver_lite::{Parser, VersionReq};

const USAGE: &str = "usage:
  semver parse <version>                 show the parts of a version
  semver sort <version>...               print versions from lowest to highest
  semver bump <major|minor|patch> <version>
  semver match <requirement> <version>...  print the versions that match
  semver max <requirement> <version>...    print the highest version that matches

Versions may start with a v, like Git tags: v1.2.3";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        // Like grep: exit code 1 means "worked, but nothing matched".
        Ok(false) => ExitCode::from(1),
        Err(err) => {
            eprintln!("error: {err}");
            // Typed errors keep their cause: print the whole chain.
            let mut cause = err.source();
            while let Some(c) = cause {
                eprintln!("  caused by: {c}");
                cause = c.source();
            }
            ExitCode::from(2)
        }
    }
}

/// Runs one command. Ok(false) means "nothing matched".
fn run(args: &[String]) -> Result<bool, Box<dyn Error>> {
    // Git tags are often written v1.2.3, so the CLI is lenient about the prefix.
    let parser = Parser::new().allow_v_prefix(true);
    let Some((command, rest)) = args.split_first() else {
        return Err(USAGE.into());
    };
    match (command.as_str(), rest) {
        ("parse", [version]) => {
            let v = parser.parse(version)?;
            let pre = if v.pre.is_empty() { "(none)" } else { v.pre.as_str() };
            println!("major: {}", v.major);
            println!("minor: {}", v.minor);
            println!("patch: {}", v.patch);
            println!("pre-release: {pre}");
        }
        ("sort", versions) if !versions.is_empty() => {
            let mut versions = parser.parse_all(versions)?;
            versions.sort();
            for v in versions {
                println!("{v}");
            }
        }
        ("bump", [part, version]) => {
            let v = parser.parse(version)?;
            let next = match part.as_str() {
                "major" => v.bump_major(),
                "minor" => v.bump_minor(),
                "patch" => v.bump_patch(),
                other => return Err(format!("cannot bump {other:?}: use major, minor or patch").into()),
            };
            println!("{next}");
        }
        ("match", [req, versions @ ..]) if !versions.is_empty() => {
            let req: VersionReq = req.parse()?;
            let versions = parser.parse_all(versions)?;
            let mut any = false;
            for v in req.matching(&versions) {
                println!("{v}");
                any = true;
            }
            return Ok(any);
        }
        ("max", [req, versions @ ..]) if !versions.is_empty() => {
            let req: VersionReq = req.parse()?;
            let versions = parser.parse_all(versions)?;
            match req.max_satisfying(&versions) {
                Some(v) => println!("{v}"),
                None => return Ok(false),
            }
        }
        _ => return Err(USAGE.into()),
    }
    Ok(true)
}
