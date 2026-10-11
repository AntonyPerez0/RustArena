#!/usr/bin/env bash
# Grades project 09: formatting, lints, the provided tests (unchanged), your new
# migration (without editing the shipped one), and the CLI storing data in a real file.
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/../../tools/lib.sh"

added_a_migration() {
  local n
  n=$(find "$HERE/migrations" -maxdepth 1 -name '*.sql' | wc -l)
  [ "$n" -ge 2 ] || { echo "Add a second migration in migrations/ (for example migrations/0002_create_warehouses_and_stock.sql)"; return 1; }
}

# Runs the CLI against a database file in a temporary folder, then opens that file with
# the sqlite3 shell to check what's really stored. The ( ) body runs in a subshell, so
# DATABASE_URL doesn't leak into other checks.
cli_uses_a_real_file() (
  local tmp out expected stored
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  export DATABASE_URL="sqlite://$tmp/inventory.db"
  run() { (cd "$HERE" && cargo run --quiet -- "$@"); }
  run add-product KB-01 Keyboard 4999 &&
    run add-warehouse Lyon &&
    run add-warehouse Paris &&
    run receive Lyon KB-01 10 &&
    run transfer KB-01 Lyon Paris 4 || return 1
  if run transfer KB-01 Lyon Paris 100 2>/dev/null; then
    echo "Transferring 100 when Lyon holds 6 should fail (exit code 1)"
    return 1
  fi
  out="$(run report)" || return 1
  expected=$'Lyon\tKB-01\t6\nParis\tKB-01\t4'
  if [ "$out" != "$expected" ]; then
    printf 'inventory report printed:\n%s\nexpected:\n%s\n' "$out" "$expected"
    return 1
  fi
  if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "The sqlite3 shell isn't installed. Run: bash tools/setup.sh"
    return 1
  fi
  stored="$(sqlite3 "$tmp/inventory.db" "SELECT name || '=' || quantity FROM stock JOIN warehouses ON warehouses.id = stock.warehouse_id ORDER BY name;")"
  if [ "$stored" != $'Lyon=6\nParis=4' ]; then
    printf 'sqlite3 found these rows in the stock table:\n%s\nexpected Lyon=6 and Paris=4\n' "$stored"
    return 1
  fi
)

check "code is formatted (cargo fmt)" fmt_ok "$HERE"
check "no clippy warnings" clippy_ok "$HERE"
check "the provided tests are unchanged" unchanged "$HERE/tests/inventory.rs" e3aff65d618e81f0849243d98d26459aebbcf9822f0ba0fe32bf0767f5642469
check "the shipped migration 0001 is unchanged (add new ones instead)" unchanged "$HERE/migrations/0001_create_products.sql" 4aea72ea137ae3d29b7765eea14af80056e62db4825bea3d3714618aa14e7aa0
check "you added a migration" added_a_migration
check "all tests pass" tests_ok "$HERE"
check "the CLI stores its data in a real SQLite file" cli_uses_a_real_file
finish
