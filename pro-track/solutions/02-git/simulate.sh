#!/usr/bin/env bash
# Maintainer check: performs every task in the project's README with plain git
# (local merges stand in for pull requests; they produce the same merge commits).
set -e
export GIT_AUTHOR_NAME=Learner GIT_AUTHOR_EMAIL=l@example.com GIT_COMMITTER_NAME=Learner GIT_COMMITTER_EMAIL=l@example.com
git init -q -b main
git add -A
git commit -qm "Import the Pro Track starter"
D=projects/02-git

# Task 2: a feature branch merged like a pull request.
git switch -qc feature/intro
printf '# About me\nI am learning Rust.\nGoal: ship production code.\n' >$D/intro.md
git add $D/intro.md
git commit -qm "Add an introduction"
git switch -q main
git merge -q --no-ff feature/intro -m "Merge branch 'feature/intro'"

# Task 3: two branches change the captain; the second merge conflicts.
git switch -qc feature/captain-a
sed -i 's/"TBD"/"ferris"/' $D/src/lib.rs
git commit -qam "Make ferris the captain"
git switch -q main
git switch -qc feature/captain-b
sed -i 's/"TBD"/"nova"/' $D/src/lib.rs
git commit -qam "Make nova the captain"
git switch -q main
git merge -q --no-ff feature/captain-a -m "Merge branch 'feature/captain-a'"
if git merge -q --no-ff feature/captain-b -m "Merge branch 'feature/captain-b'" >/dev/null 2>&1; then
  echo "expected a merge conflict" >&2
  exit 1
fi
cat >$D/src/lib.rs <<'EOF'
//! The team roster for the next tournament.
//!
//! Two teammates are about to change this file at the same time: see task 3 in
//! the README.

/// Everyone on the team, in the order they joined.
pub const PLAYERS: [&str; 5] = ["ferris", "nova", "kestrel", "atlas", "pixel"];

/// The most players a team may field in a match.
pub const MAX_PLAYERS: usize = 5;

/// Who leads the team. Must be one of [`PLAYERS`].
pub fn captain() -> &'static str {
    "ferris"
}

/// Who leads when the captain can't.
pub fn vice_captain() -> &'static str {
    "nova"
}

/// One line for the team page.
pub fn summary() -> String {
    format!("{} players, captain: {}", PLAYERS.len(), captain())
}
EOF
git add $D/src/lib.rs
git commit -qm "Merge branch 'feature/captain-b'"

# Task 4: a bad commit on main, undone with git revert.
sed -i 's/MAX_PLAYERS: usize = 5/MAX_PLAYERS: usize = 6/' $D/src/lib.rs
git commit -qam "Allow six players"
git revert --no-edit HEAD >/dev/null

# Task 5: bump the version and tag the release.
sed -i 's/^version = "0.0.0"/version = "0.1.0"/' $D/Cargo.toml
sed -i '/^name = "roster"$/{n;s/^version = "0.0.0"/version = "0.1.0"/}' $D/Cargo.lock
git commit -qam "Release 0.1.0"
git tag -a v0.1.0 -m "roster 0.1.0"
