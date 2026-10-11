#!/usr/bin/env bash
# Installs what the Pro Track needs: Rust (via rustup) with rustfmt and clippy, plus Git and
# SQLite. Safe to run again. Works on GitHub Codespaces, Ubuntu/Debian and macOS.
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  if [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "Installing Rust with rustup (https://rustup.rs)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
fi

rustup toolchain install stable --profile minimal --component rustfmt,clippy >/dev/null
rustup default stable >/dev/null

if command -v apt-get >/dev/null 2>&1; then
  SUDO=""
  if [ "$(id -u)" -ne 0 ]; then SUDO="sudo"; fi
  $SUDO apt-get update -qq
  $SUDO apt-get install -y -qq git sqlite3 build-essential pkg-config >/dev/null
elif command -v brew >/dev/null 2>&1; then
  brew install git sqlite >/dev/null || true
fi

echo
bash "$(dirname "$0")/doctor.sh" || true
echo
echo "If 'cargo' isn't found in this terminal, open a new terminal (or run: . \"\$HOME/.cargo/env\")."
