#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
install_dir="${ELROI_BIN_DIR:-$HOME/.local/bin}"
applications_dir="${ELROI_APPLICATIONS_DIR:-$HOME/Applications}"
skip_desktop=0

usage() {
  cat <<'EOF'
Usage: scripts/install-elroi.sh [--cli-only]

Builds and installs the ElRoi CLI and desktop app from this source checkout.

Environment overrides:
  ELROI_BIN_DIR          CLI install directory (default: ~/.local/bin)
  ELROI_APPLICATIONS_DIR Desktop app install directory (default: ~/Applications)

Outputs:
  elroi CLI:            $ELROI_BIN_DIR/elroi
  desktop app:          $ELROI_APPLICATIONS_DIR/ElRoi.app on macOS
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cli-only)
      skip_desktop=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

cd "$repo_root"

export HERMIT_ENV="$repo_root"
export HERMIT_BIN="$repo_root/bin"
export CARGO_HOME="$repo_root/.hermit/rust"
export COREPACK_HOME="$repo_root/.hermit/node"
export NPM_CONFIG_CACHE="$repo_root/.hermit/node/cache"
export NPM_CONFIG_PREFIX="$repo_root/.hermit/node"
export CI="${CI:-true}"
export PNPM_CONFIG_CONFIRM_MODULES_PURGE="${PNPM_CONFIG_CONFIRM_MODULES_PURGE:-false}"
export PATH="$repo_root/.hermit/rust/bin:$repo_root/.hermit/pnpm:$repo_root/node_modules/.bin:$repo_root/.hermit/node/bin:$repo_root/bin:$PATH"

echo "Building ElRoi CLI..."
cargo build --release -p goose-cli --bin elroi

mkdir -p "$install_dir"
install -m 755 "$repo_root/target/release/elroi" "$install_dir/elroi"
echo "Installed CLI: $install_dir/elroi"

mkdir -p "$repo_root/ui/desktop/src/bin"
install -m 755 "$repo_root/target/release/elroi" "$repo_root/ui/desktop/src/bin/elroi"
echo "Staged desktop backend: ui/desktop/src/bin/elroi"

if [[ "$skip_desktop" -eq 1 ]]; then
  exit 0
fi

echo "Installing desktop dependencies..."
(cd "$repo_root/ui/desktop" && pnpm install)

echo "Packaging ElRoi desktop app..."
(cd "$repo_root/ui/desktop" && pnpm run package)

case "$(uname -s)" in
  Darwin)
    arch_name="$(uname -m)"
    case "$arch_name" in
      arm64) electron_arch="arm64" ;;
      x86_64) electron_arch="x64" ;;
      *) echo "Unsupported macOS architecture: $arch_name" >&2; exit 1 ;;
    esac

    app_path="$repo_root/ui/desktop/out/ElRoi-darwin-$electron_arch/ElRoi.app"
    if [[ ! -d "$app_path" ]]; then
      echo "Packaged app not found at $app_path" >&2
      exit 1
    fi

    mkdir -p "$applications_dir"
    rm -rf "$applications_dir/ElRoi.app"
    cp -R "$app_path" "$applications_dir/ElRoi.app"
    echo "Installed desktop app: $applications_dir/ElRoi.app"
    ;;
  *)
    echo "Desktop package created under ui/desktop/out."
    echo "Automatic app copy is currently implemented for macOS only."
    ;;
esac
