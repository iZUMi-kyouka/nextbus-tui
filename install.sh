#!/usr/bin/env bash
set -euo pipefail

REPO="${NEXTBUS_REPO:-iZUMi-kyouka/nextbus-tui}"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
USER_AGENT="nextbus-tui-installer"
DRY_RUN=0
USE_SUDO_INSTALL=0

usage() {
  cat <<'EOF'
Usage: bash install.sh [--dry-run] [--help]

Downloads the latest nextbus-tui release binary for your OS/arch,
installs it to ~/.local/bin or /usr/local/bin, and offers launcher/alias setup.

Environment variables:
  NEXTBUS_REPO   GitHub repo in owner/name format (default: iZUMi-kyouka/nextbus-tui)
EOF
}

log() {
  printf '[installer] %s\n' "$*"
}

die() {
  printf '[installer] ERROR: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

confirm() {
  local prompt="$1"
  local default_yes="${2:-1}"
  local suffix='[y/N]'
  if [[ "$default_yes" == "1" ]]; then
    suffix='[Y/n]'
  fi

  while true; do
    read -r -p "$prompt $suffix " answer
    if [[ -z "$answer" ]]; then
      [[ "$default_yes" == "1" ]] && return 0 || return 1
    fi
    case "$answer" in
      y|Y|yes|YES) return 0 ;;
      n|N|no|NO) return 1 ;;
      *) log "Please answer y or n." ;;
    esac
  done
}

run() {
  if [[ "$DRY_RUN" == "1" ]]; then
    log "DRY RUN: $*"
  else
    "$@"
  fi
}

run_sudo() {
  if [[ "$DRY_RUN" == "1" ]]; then
    log "DRY RUN: sudo $*"
  else
    sudo "$@"
  fi
}

pick_asset() {
  local os="$1"
  local arch="$2"
  local json_payload="$3"

  python3 - "$os" "$arch" "$json_payload" <<'PY'
import json
import re
import sys

os_name = sys.argv[1]
arch = sys.argv[2]
release = json.loads(sys.argv[3])
assets = release.get("assets", [])
name_to_url = {a.get("name", ""): a.get("browser_download_url", "") for a in assets}

if os_name == "linux":
    candidates = [
        f"nextbus-tui-linux-{arch}.tar.gz",
        f"nextbus-tui-linux-{arch}",
    ]
elif os_name == "macos":
    arch_aliases = [arch]
    if arch == "aarch64":
        arch_aliases.append("arm64")
    elif arch == "x86_64":
        arch_aliases.append("amd64")

    candidates = []
    for alias in arch_aliases:
        candidates.extend([
            f"nextbus-tui-macos-{alias}.tar.gz",
            f"nextbus-tui-macos-{alias}",
        ])
else:
    print("Unsupported OS for this installer", file=sys.stderr)
    sys.exit(2)

for c in candidates:
    if c in name_to_url:
        print(c + "\t" + name_to_url[c])
        sys.exit(0)

# Regex fallback for future naming tweaks.
patterns = [
    rf"^nextbus-tui-{os_name}-.*{arch}.*(?:\\.tar\\.gz)?$",
    rf"^nextbus-tui-{os_name}-.*(?:\\.tar\\.gz)?$",
]
for p in patterns:
    regex = re.compile(p)
    for name, url in name_to_url.items():
        if regex.match(name):
            print(name + "\t" + url)
            sys.exit(0)

print("No matching release asset found for this OS/arch.", file=sys.stderr)
print("Available assets:", file=sys.stderr)
for a in sorted(name_to_url):
    print("  - " + a, file=sys.stderr)
sys.exit(1)
PY
}

for arg in "$@"; do
  case "$arg" in
    --help|-h)
      usage
      exit 0
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    *)
      die "Unknown argument: $arg"
      ;;
  esac
done

need_cmd curl
need_cmd python3
need_cmd uname

case "$(uname -s)" in
  Linux) OS="linux" ;;
  Darwin) OS="macos" ;;
  *) die "This installer supports Linux and macOS. Use install.ps1 on Windows." ;;
esac

case "$(uname -m)" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) die "Unsupported architecture: $(uname -m)" ;;
esac

log "Detected OS=$OS ARCH=$ARCH"
log "Resolving latest release from $REPO"

release_json="$(curl -fsSL -H "User-Agent: $USER_AGENT" "$API_URL")"
asset_line="$(pick_asset "$OS" "$ARCH" "$release_json")"
asset_name="${asset_line%%$'\t'*}"
asset_url="${asset_line#*$'\t'}"

log "Selected asset: $asset_name"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

download_path="$tmp_dir/$asset_name"
if [[ "$DRY_RUN" == "1" ]]; then
  log "DRY RUN: curl -fL '$asset_url' -o '$download_path'"
else
  curl -fL "$asset_url" -o "$download_path"
fi

binary_source=""
if [[ "$asset_name" == *.tar.gz ]]; then
  need_cmd tar
  run tar -xzf "$download_path" -C "$tmp_dir"
  binary_source="$(find "$tmp_dir" -type f -name nextbus-tui | head -n 1 || true)"
elif [[ "$asset_name" == *.zip ]]; then
  need_cmd unzip
  run unzip -q "$download_path" -d "$tmp_dir"
  binary_source="$(find "$tmp_dir" -type f -name nextbus-tui | head -n 1 || true)"
else
  binary_source="$download_path"
fi

[[ -n "$binary_source" ]] || die "Failed to locate nextbus-tui binary in downloaded asset"

install_dir="${HOME}/.local/bin"
if confirm "Install binary to /usr/local/bin (system-wide)?" 0; then
  install_dir="/usr/local/bin"
  USE_SUDO_INSTALL=1
fi

install_path="${install_dir}/nextbus-tui"
if [[ "$USE_SUDO_INSTALL" == "1" ]]; then
  run_sudo mkdir -p "$install_dir"
  run_sudo install -m 0755 "$binary_source" "$install_path"
else
  run mkdir -p "$install_dir"
  run install -m 0755 "$binary_source" "$install_path"
fi

log "Installed binary to $install_path"

if confirm "Also create terminal command alias 'nnbus'?" 1; then
  alias_path="${install_dir}/nnbus"
  if [[ "$USE_SUDO_INSTALL" == "1" ]]; then
    run_sudo ln -sf "$install_path" "$alias_path"
  else
    run ln -sf "$install_path" "$alias_path"
  fi
  log "Installed terminal alias: $alias_path -> $install_path"
fi

if [[ "$OS" == "linux" ]]; then
  if confirm "Install .desktop launcher for app menu and double-click support?" 1; then
    applications_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
    desktop_file="${applications_dir}/nextbus-tui.desktop"
    run mkdir -p "$applications_dir"

    if [[ "$DRY_RUN" == "1" ]]; then
      log "DRY RUN: write $desktop_file"
    else
      cat > "$desktop_file" <<EOF
[Desktop Entry]
Type=Application
Name=nextbus-tui
Comment=NUS shuttle bus arrival TUI
Exec=$install_path
Terminal=true
Categories=Utility;
EOF
      chmod 0644 "$desktop_file"
    fi

    if [[ -d "$HOME/Desktop" ]] && confirm "Copy launcher to Desktop too?" 0; then
      run cp "$desktop_file" "$HOME/Desktop/nextbus-tui.desktop"
      run chmod 0755 "$HOME/Desktop/nextbus-tui.desktop"
    fi

    log "Linux launcher created. You can search for 'nextbus-tui' from your app menu."
  fi
fi

if [[ "$OS" == "macos" ]]; then
  if confirm "Install Finder launcher (.command) for double-click support?" 1; then
    launcher_target=""
    if confirm "Install launcher in /Applications? (requires sudo permissions)" 0; then
      launcher_target="/Applications/nextbus-tui.command"
      if [[ "$DRY_RUN" == "1" ]]; then
        log "DRY RUN: write $launcher_target with sudo"
      else
        cat <<EOF | sudo tee "$launcher_target" >/dev/null
#!/usr/bin/env bash
exec "$install_path"
EOF
        sudo chmod 0755 "$launcher_target"
      fi
    else
      run mkdir -p "$HOME/Applications"
      launcher_target="$HOME/Applications/nextbus-tui.command"
      if [[ "$DRY_RUN" == "1" ]]; then
        log "DRY RUN: write $launcher_target"
      else
        cat > "$launcher_target" <<EOF
#!/usr/bin/env bash
exec "$install_path"
EOF
        chmod 0755 "$launcher_target"
      fi
    fi

    if [[ "$DRY_RUN" != "1" ]]; then
      xattr -d com.apple.quarantine "$install_path" 2>/dev/null || true
      xattr -d com.apple.quarantine "$launcher_target" 2>/dev/null || true
    fi

    log "macOS launcher created at $launcher_target"
  fi
fi

log "Install completed."
log "Run: $install_path"




