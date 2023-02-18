#!/usr/bin/env -S just --justfile

# Cross platform shebang:
shebang := if os() == 'windows' {
  'pwsh.exe'
} else {
  '/usr/bin/env bash'
}

set shell := ["/usr/bin/env", "bash" ,"-c"]
set windows-shell := ["pwsh.exe","-NoLogo", "-noprofile", "-c"]

originalExe := "ncm-rs"
newExe := "ncm"
targetPath := "./target/release/"
releasePath := "./target/release/ncm"
installPath :=  "/.local/bin/"

lint:
  cargo clippy

test:
  cargo test

debug:
    cargo build

release: test
  cargo build --release

# --| Build ----------------------
# --|-----------------------------
build:
  just _build-{{os()}}

# -| Linux
_build-linux: release
  rm ./target/release/ncm || true
  mv -f ./target/release/ncm-rs ./target/release/ncm || true

# --| Windows
_build-windows: release
  if (test-path "{{releasePath}}.exe") { rm "{{releasePath}}.exe" };
  mv '{{targetPath}}{{originalExe}}.exe' '{{targetPath}}{{newExe}}.exe'

# --| MacOS
_build-macos: release

# --| Install --------------------
# Install NCM --------------------
install:
	just _install-{{os()}}

# --| Linux
_install-linux: build
  cp ./target/release/ncm $HOME/.local/bin

# --| Windows
_install-windows: build
  cp -force '{{releasePath}}.exe' ${HOME}/'{{installPath}}'

# --| MacOS
_install-macos: build

# --| Manual Cleanup -------------
# Revert all changes to system ---
reset:
  test -L ~/.config/nvim && rm ~/.config/nvim || true
  mv ~/.config/nvim_configs/nvim ~/.config/ || true
  rm -rf ~/.config/nvim_configs || true
  rm -rf ~/.config/ncm-rs || true

# --| Reset To Default and Rebuild/Install NCM
reinstall: reset install
