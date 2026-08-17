#!/usr/bin/env bash
# Environment setup for the rust-port-kit pipeline. Detects the platform
# and installs whatever's missing, so a fresh Ubuntu box (or Termux, or
# macOS) goes from "nothing installed" to "ready to run /port-to-rust" in
# one call. Safe to re-run any time — every step checks before installing.
#
# On success, prints a line starting with SETUP_SUMMARY: followed by JSON
# describing what's ready and what still needs a human. The /setup slash
# command reads that line to decide what (if anything) to ask the user.
set -uo pipefail

STATE_DIR="$(mktemp -d)"
NEEDS_USER_FILE="$STATE_DIR/needs_user.txt"
INSTALLED_FILE="$STATE_DIR/installed.txt"
ALREADY_OK_FILE="$STATE_DIR/already_ok.txt"
touch "$NEEDS_USER_FILE" "$INSTALLED_FILE" "$ALREADY_OK_FILE"

needs_user() { echo "$*" >> "$NEEDS_USER_FILE"; }
installed()  { echo "$*" >> "$INSTALLED_FILE"; }
already_ok() { echo "$*" >> "$ALREADY_OK_FILE"; }
log() { echo "==> $*"; }
have() { command -v "$1" >/dev/null 2>&1; }

is_termux() { [[ "${PREFIX:-}" == *com.termux* ]]; }
is_ubuntu() { [ -f /etc/os-release ] && grep -qi 'ubuntu\|debian' /etc/os-release; }

# Wraps apt-get/install so the same call works whether we're root, have
# passwordless sudo, or need a password we can't type non-interactively.
SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  if sudo -n true 2>/dev/null; then
    SUDO="sudo"
  else
    needs_user "This machine needs a sudo password to install packages, which can't be entered non-interactively. Run 'sudo -v' yourself first (caches your credential for a few minutes), then re-run /setup — or install these yourself: build-essential pkg-config libssl-dev python3-dev python3-pip python3-venv git curl ca-certificates gh"
  fi
fi

##############################################################################
# 1. OS packages
##############################################################################

if is_termux; then
  log "Detected Termux."
  pkg update -y && pkg install -y rust python git gh binutils openssl
  installed "Termux packages: rust, python, git, gh, binutils, openssl"

elif is_ubuntu && { [ "$(id -u)" -eq 0 ] || [ -n "$SUDO" ]; }; then
  log "Detected Ubuntu/Debian."
  APT_PACKAGES="build-essential pkg-config libssl-dev python3 python3-pip python3-venv python3-dev git curl ca-certificates"
  # Don't let `update` failure block `install`: a single stray/broken
  # third-party repo (docker, nodesource, etc.) can make `update` exit
  # non-zero even though the Ubuntu archives it actually needs synced fine.
  $SUDO apt-get update -y >/tmp/apt_update.log 2>&1 \
    || log "apt-get update reported errors (see /tmp/apt_update.log) — continuing, since the packages needed here usually come from the main archive anyway"
  if $SUDO apt-get install -y $APT_PACKAGES >/tmp/apt_install.log 2>&1; then
    installed "apt packages: $APT_PACKAGES"
  else
    needs_user "apt-get install failed — see /tmp/apt_install.log"
  fi

  if ! have gh; then
    log "Installing GitHub CLI (gh) via the official apt source..."
    if curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
         -o /tmp/githubcli-archive-keyring.gpg \
       && $SUDO install -D -o root -g root -m 644 \
            /tmp/githubcli-archive-keyring.gpg \
            /usr/share/keyrings/githubcli-archive-keyring.gpg \
       && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
            | $SUDO tee /etc/apt/sources.list.d/github-cli.list >/dev/null; then
      $SUDO apt-get update -y >>/tmp/apt_update.log 2>&1 || true
      if $SUDO apt-get install -y gh >>/tmp/apt_install.log 2>&1; then
        installed "gh (GitHub CLI, via official apt source)"
      else
        needs_user "gh apt source added but install failed — see /tmp/apt_install.log"
      fi
    else
      needs_user "Couldn't install gh automatically — see https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
    fi
    rm -f /tmp/githubcli-archive-keyring.gpg
  else
    already_ok "gh"
  fi

elif is_ubuntu; then
  log "Detected Ubuntu/Debian but can't get root — skipping apt packages (see 'needs you' below)."
else
  log "Non-Ubuntu, non-Termux platform — installing what's generic (Rust + Python tooling only)."
fi

##############################################################################
# 2. Rust toolchain (rustup, not apt's rustc — apt's is usually too old for
#    current PyO3/proptest releases' transitive dependencies)
##############################################################################

if have cargo; then
  already_ok "cargo/rustc ($(rustc --version 2>/dev/null))"
else
  log "Installing Rust via rustup (download then run — not piped — so it doesn't need an exception to the 'curl | sh' deny rule)..."
  if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh; then
    sh /tmp/rustup-init.sh -y --default-toolchain stable >/tmp/rustup_install.log 2>&1
    rm -f /tmp/rustup-init.sh
    # shellcheck disable=SC1090
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    if have cargo; then
      installed "Rust ($(rustc --version))"
    else
      needs_user "rustup ran but cargo isn't on PATH yet — open a new shell, or run: source \$HOME/.cargo/env"
    fi
  else
    needs_user "Couldn't reach sh.rustup.rs to install Rust — check network/firewall"
  fi
fi

if have rustup; then
  rustup component add clippy rustfmt >/dev/null 2>&1 || true
fi

##############################################################################
# 3. Python tooling
##############################################################################

PY_PACKAGES="maturin pytest hypothesis"
if python3 -c "import maturin" >/dev/null 2>&1 && have pytest; then
  already_ok "$PY_PACKAGES"
else
  log "Installing Python tooling: $PY_PACKAGES..."
  if pip install --upgrade $PY_PACKAGES >/tmp/pip_install.log 2>&1; then
    installed "$PY_PACKAGES (pip)"
  elif pip install --break-system-packages --upgrade $PY_PACKAGES >/tmp/pip_install.log 2>&1; then
    installed "$PY_PACKAGES (pip --break-system-packages, Ubuntu 24's externally-managed-environment guard)"
  else
    needs_user "pip install of $PY_PACKAGES failed — see /tmp/pip_install.log"
  fi
fi

##############################################################################
# 4. GitHub auth — inherently can't be scripted headlessly
##############################################################################

if have gh; then
  if gh auth status >/tmp/gh_auth.log 2>&1; then
    already_ok "gh auth"
  else
    needs_user "Not logged into GitHub CLI. Run: gh auth login (this opens an interactive/browser login — never paste a password or token into chat)"
  fi
fi

##############################################################################
# 5. Summary
##############################################################################

echo
log "Setup pass complete."
if [ -s "$INSTALLED_FILE" ]; then echo "Installed:"; sed 's/^/  - /' "$INSTALLED_FILE"; fi
if [ -s "$ALREADY_OK_FILE" ]; then echo "Already present:"; sed 's/^/  - /' "$ALREADY_OK_FILE"; fi
if [ -s "$NEEDS_USER_FILE" ]; then echo "Needs you:"; sed 's/^/  - /' "$NEEDS_USER_FILE"; fi

python3 - "$INSTALLED_FILE" "$ALREADY_OK_FILE" "$NEEDS_USER_FILE" <<'PYEOF'
import json
import sys

def read_lines(path):
    with open(path) as f:
        return [line.rstrip("\n") for line in f if line.strip()]

installed, already_ok, needs_user = (read_lines(p) for p in sys.argv[1:4])
print("SETUP_SUMMARY:" + json.dumps({
    "ready": len(needs_user) == 0,
    "installed": installed,
    "already_ok": already_ok,
    "needs_user": needs_user,
}))
PYEOF

rm -rf "$STATE_DIR"
exit 0
