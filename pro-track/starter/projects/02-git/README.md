# 02 · Git and pull requests

Every professional team uses Git, and almost all of them work through **pull requests** (PRs): you change code on a branch, open a PR, the CI checks run, a teammate reviews it, and then it's merged. This project walks you through that whole loop on a small Rust crate, including the parts that scare beginners: a merge conflict in Rust code, undoing a commit that broke the build, and tagging a release.

There's little code to write. The grader reads your repository's **history**, then checks that the merged code still compiles, is formatted, and passes its tests.

**You'll practice:** commits and commit messages, branches, pull requests, merge conflicts, `git revert`, semantic versioning and annotated tags.

## Background

### The mental model

- A **commit** is a snapshot of the whole project, plus a message and a pointer to its parent commit(s).
- A **branch** is just a movable name pointing at a commit. `main` is the shared branch; you work on your own.
- A **merge commit** has two parents and joins two lines of history.
- A **remote** (`origin`) is the copy on GitHub. `git push` sends your commits there; `git pull` brings other people's here.

```bash
git status                        # what changed?
git diff                          # exactly how?
git add path/to/file              # stage it for the next commit
git commit -m "Add the parser"    # take the snapshot
git log --oneline --graph --all   # the history as a graph
```

### Commit messages

Teams read history all the time (`git log`, `git blame`, release notes), so messages follow conventions: a **subject line** of at most 72 characters (50 is better), in the **imperative** ("Add parser", not "Added parser"), starting with a capital letter, with **no trailing period**. If it needs explaining, add a blank line and a body that says **why**; the diff already shows what.

### Pull requests

A PR asks "please merge my branch into `main`". It shows the diff, runs CI (here, the **Grade** workflow) and collects review comments. Keep PRs small and focused, describe what changed and how you tested it, and only merge when the checks are green. Teams usually protect `main` so a PR is the only way in.

### Merge conflicts

When two branches change the **same lines**, Git can't decide which version wins and stops with markers in the file:

```rust
pub fn captain() -> &'static str {
<<<<<<< HEAD
    "ferris"
=======
    "nova"
>>>>>>> feature/captain-b
}
```

You edit the file into the version you want (often a mix of both), delete every marker line, then `git add` and `git commit` to finish the merge. In Rust, **always run `cargo test` before committing the resolution**: a leftover marker doesn't compile, and two changes that merge without a conflict can still break each other (one branch renames a function that the other starts calling). That's why CI runs again after every merge.

### Undoing things safely

| Situation | Command |
|---|---|
| Unstage a file | `git restore --staged file` |
| Throw away uncommitted edits | `git restore file` |
| Undo a commit that's already pushed | `git revert <commit>`: a **new** commit that reverses it |
| Rewrite commits you haven't pushed yet | `git commit --amend`, `git rebase -i` |

Never rewrite history other people already have (no `git push --force` on `main`). `revert` is the safe tool once something is public.

### Releases and tags

Rust crates use **semantic versioning** (`MAJOR.MINOR.PATCH`) in `Cargo.toml`. A release is a commit that sets the version, marked with a **tag**. Use an **annotated** tag (`git tag -a`): it records who tagged, when and why, like a commit does. A plain `git tag` is just a name.

## Your tasks

Run the commands from the repository root. You can use the GitHub website for the PR steps.

1. **Set your identity** (once per machine):
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "you@example.com"
   ```
2. **A feature branch and a pull request.**
   ```bash
   git switch -c feature/intro
   # create projects/02-git/intro.md with at least 3 lines about you
   git add projects/02-git/intro.md
   git commit -m "Add an introduction"
   git push -u origin feature/intro
   ```
   On GitHub, open **Pull requests > New pull request**, pick `feature/intro`, create it, look at the checks, then **Merge pull request** with *Create a merge commit*. Back in the terminal: `git switch main && git pull`.
3. **Make (and fix) a merge conflict.** Two teammates pick a captain in `src/lib.rs` at the same time:
   ```bash
   git switch -c feature/captain-a
   # in projects/02-git/src/lib.rs, change "TBD" to one name from PLAYERS
   git commit -am "Make ferris the captain"
   git push -u origin feature/captain-a
   git switch main
   git switch -c feature/captain-b
   # change "TBD" to a DIFFERENT name from PLAYERS
   git commit -am "Make nova the captain"
   git push -u origin feature/captain-b
   ```
   Open and merge a PR for `feature/captain-a`. Then open a PR for `feature/captain-b`: GitHub reports a conflict. Resolve it locally:
   ```bash
   git fetch origin
   git switch feature/captain-b
   git merge origin/main             # CONFLICT in projects/02-git/src/lib.rs
   # edit the file: keep one captain, and add a vice_captain() function
   # returning the other name, if you like; delete the marker lines
   cargo test --manifest-path projects/02-git/Cargo.toml
   git add projects/02-git/src/lib.rs
   git commit                        # finishes the merge
   git push
   ```
   Merge the PR once it's green, then `git switch main && git pull`.
4. **Undo a mistake with revert.** On `main`, set `MAX_PLAYERS` to `6`, commit it as "Allow six players" and push. The Grade check for 02-git fails because a test enforces the rules. Undo it the safe way:
   ```bash
   git revert HEAD
   git push
   ```
5. **Release 0.1.0.** In `projects/02-git/Cargo.toml` change `version = "0.0.0"` to `"0.1.0"`, run `cargo build` in that folder so `Cargo.lock` picks it up, and commit both files as "Release 0.1.0". Then:
   ```bash
   git tag -a v0.1.0 -m "roster 0.1.0"
   git push origin main v0.1.0
   ```
6. **Check yourself:** `bash tools/grade.sh 02-git`.

## What the grader checks

- `intro.md` exists with at least 3 non-empty lines,
- at least three `feature/` branches were merged (PR merge commits or `git merge`),
- a merge commit joins two branches that both changed `src/lib.rs`, and no conflict markers are left,
- `tests/roster.rs` is unchanged, and the crate passes `cargo fmt --check`, clippy and its tests,
- a `Revert "..."` commit touches this project,
- `v0.1.0` is an annotated tag in your history, on a commit whose `Cargo.toml` says `version = "0.1.0"`,
- your commit subjects in this project are at most 72 characters, start with a capital letter and don't end with a period.

## Hints

- Lost? `git log --oneline --graph --all` shows where every branch points. `git merge --abort` gets you out of a merge you want to restart.
- A lightweight tag fails the check. Delete it (`git tag -d v0.1.0`, plus `git push origin :refs/tags/v0.1.0` if pushed) and use `-a`.
- Your editor or VS Code shows *Accept Current / Incoming / Both* buttons on conflicts. Handy, but read the result before committing.

## Going further

- Protect `main` (**Settings > Branches**): require pull requests and a passing Grade check. That's how most companies configure it.
- Try `git rebase main` on a branch instead of merging, and compare the graph.
- Find the commit that broke a test with `git bisect run cargo test`.
- Turn your tag into a **GitHub Release** with notes (Releases > Draft a new release).
