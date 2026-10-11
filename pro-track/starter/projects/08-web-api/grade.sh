#!/usr/bin/env bash
# Grades project 08: the bookmarks REST API passes the provided endpoint tests,
# which call the router in-process with tower's oneshot (no port, no network).
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

# code_has <file> <message> <regex>: the regex appears in the file (comments ignored).
code_has() {
  sed 's://.*$::' "$HERE/$1" | grep -Eq "$3" || { echo "$2"; return 1; }
}

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings" clippy_ok "$HERE"
check "provided tests are unchanged" unchanged "$HERE/tests/api.rs" 9705d80d24eccd76d374541f695cba61fc69a03541e8af5f469a5f7d1df18e39
check "AppError implements IntoResponse" code_has src/error.rs \
  "Implement IntoResponse for AppError in src/error.rs" 'impl +IntoResponse +for +AppError'
check "routes use axum 0.8 path parameters (/{id})" code_has src/routes.rs \
  "Add the /bookmarks/{id} route in src/routes.rs" '"/bookmarks/\{id\}"'
check "all endpoint tests pass" tests_ok "$HERE"
finish
