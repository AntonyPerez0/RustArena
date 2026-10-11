#!/usr/bin/env bash
# Grades project 02 by reading your repository's history, then checks that the
# merged Rust code still builds cleanly and passes its tests.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"
cd "$ARENA_ROOT" || exit 1
D=projects/02-git
LIB=$D/src/lib.rs

in_repo() {
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "run this inside your clone of the repository"; return 1; }
}

intro_ok() {
  [ -f "$D/intro.md" ] || { echo "$D/intro.md is missing"; return 1; }
  local lines
  lines=$(grep -c '[^[:space:]]' "$D/intro.md")
  [ "$lines" -ge 3 ] || { echo "intro.md needs at least 3 non-empty lines (has $lines)"; return 1; }
}

# Counts distinct feature/ branches that were merged, from GitHub's
# "Merge pull request #N from you/feature/x" or git's "Merge branch 'feature/x'".
feature_merges() {
  local names n
  names=$(git log --merges --format=%s | grep -oE "(from [^ /]+/|Merge branch ')feature/[^ ']+" |
    sed -E 's/^.*(feature\/)/\1/' | sort -u)
  n=$(printf '%s' "$names" | grep -c .)
  echo "feature branches merged: $n"
  [ -n "$names" ] && printf '  %s\n' $names
  [ "$n" -ge 3 ] || { echo "merge at least three feature/ branches (tasks 2 and 3)"; return 1; }
}

# A real conflict: a merge commit whose two parents both changed src/lib.rs
# (compared with where they split), in different ways.
conflict_ok() {
  if grep -qE '^(<<<<<<<|=======|>>>>>>>)' "$LIB"; then
    echo "$LIB still has conflict markers"
    return 1
  fi
  local m p1 p2 base b
  for m in $(git rev-list --merges HEAD); do
    p1=$(git rev-parse -q --verify "$m^1:$LIB") || continue
    p2=$(git rev-parse -q --verify "$m^2:$LIB") || continue
    base=$(git merge-base "$m^1" "$m^2") || continue
    b=$(git rev-parse -q --verify "$base:$LIB") || continue
    if [ "$p1" != "$b" ] && [ "$p2" != "$b" ] && [ "$p1" != "$p2" ]; then
      echo "resolved in: $(git log -1 --format='%h %s' "$m")"
      return 0
    fi
  done
  echo "no merge commit joins two branches that both changed src/lib.rs (task 3)"
  return 1
}

revert_ok() {
  git log --format=%s -- "$D" | grep -q '^Revert "' ||
    { echo 'no commit made with git revert (subject starting with Revert ") touches this project'; return 1; }
}

tag_ok() {
  [ "$(git cat-file -t v0.1.0 2>/dev/null)" = "tag" ] ||
    { echo "v0.1.0 must exist and be an annotated tag (git tag -a v0.1.0 -m ...)"; return 1; }
  git merge-base --is-ancestor v0.1.0 HEAD ||
    { echo "v0.1.0 tags a commit that isn't in this branch's history"; return 1; }
  git show "v0.1.0:$D/Cargo.toml" | grep -qE '^version *= *"0\.1\.0"' ||
    { echo "at the tagged commit, Cargo.toml's version isn't \"0.1.0\": bump it, commit, then tag"; return 1; }
}

messages_ok() {
  local bad=0 s
  while IFS= read -r s; do
    [ -z "$s" ] && continue
    case "$s" in Merge* | Revert*) continue ;; esac
    if [ ${#s} -gt 72 ]; then echo "too long (${#s} chars): $s"; bad=1; fi
    case "$s" in *.) echo "ends with a period: $s"; bad=1 ;; esac
    case "$s" in [a-z]*) echo "should start with a capital letter: $s"; bad=1 ;; esac
  done < <(git log --no-merges --format=%s -- "$D" | sed '$d')
  return $bad
}

check "this folder is a Git repository" in_repo
check "intro.md was added" intro_ok
check "three feature/ branches were merged (pull requests or git merge)" feature_merges
check "the merge conflict in src/lib.rs was resolved" conflict_ok
check "the provided tests (tests/roster.rs) are unchanged" \
  unchanged "$D/tests/roster.rs" 307003e5085dd719dbdd101ee0e627b50e4ed4b954ee1069608c290c94a028ba
check "the merged code is formatted (cargo fmt --check)" fmt_ok "$D"
check "no clippy warnings (cargo clippy -- -D warnings)" clippy_ok "$D"
check "every test passes (a real captain, and the bad commit undone)" tests_ok "$D"
check "a bad commit was undone with git revert" revert_ok
check "release tag v0.1.0 is annotated and matches Cargo.toml's version" tag_ok
check "commit messages follow the conventions" messages_ok
finish
