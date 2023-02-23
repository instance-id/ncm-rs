#!/usr/bin/env -S just --justfile

# Cross platform shebang:
shebang := if os() == 'windows' {
  'pwsh.exe'
} else {
  '/usr/bin/env bash'
}

set shell := ["/usr/bin/env", "bash" ,"-c"]
set windows-shell := ["pwsh.exe","-NoLogo", "-noprofile", "-c"]

newExe := "ncm"
originalExe := "ncm-rs"
configPath := "nvim-ncm"
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
	just _reset-{{os()}}

# @formatter:off
# --| Linux 
_reset-linux:
  test -L ~/.cache/nvim       && rm ~/.cache/nvim       || true
  test -L ~/.config/nvim      && rm ~/.config/nvim      || true
  test -L ~/.local/share/nvim && rm ~/.local/share/nvim || true

  mv "$HOME/.cache/nvim-ncm/main"       "$HOME/.cache/nvim-ncm/nvim"       && mv "$HOME/.cache/nvim-ncm/nvim"       ~/.cache/       || true
  mv "$HOME/.config/nvim-ncm/main"      "$HOME/.config/nvim-ncm/nvim"      && mv "$HOME/.config/nvim-ncm/nvim"      ~/.config/      || true
  mv "$HOME/.local/share/nvim-ncm/main" "$HOME/.local/share/nvim-ncm/nvim" && mv "$HOME/.local/share/nvim-ncm/nvim" ~/.local/share/ || true

  rm -rf "$HOME/.cache/nvim-ncm"       || true
  rm -rf "$HOME/.config/nvim-ncm"      || true
  rm -rf "$HOME/.local/share/nvim-ncm" || true

  rm -rf $HOME/.config/ncm-rs || true
  echo "Reset Complete! {{configPath}}"

# --| Windows
_reset-windows:
  #!{{shebang}}
  $cfg = if($env:XDG_CONFIG_HOME -ne $null) { $env:XDG_CONFIG_HOME } else { $env:LOCALAPPDATA }
  $dataDir = if($env:XDG_DATA_HOME -ne $null) { $env:XDG_DATA_HOME } else { $env:LOCALAPPDATA }
  $cacheDir = if($env:XDG_CACHE_HOME -ne $null) { $env:XDG_CACHE_HOME } else { $env:LOCALAPPDATA }
  #
  $nvimDir = "${cfg}\nvim"
  $nvimData = "${dataDir}\nvim-data"
  $nvimCache = "${cacheDir}\nvim"
  echo "nvimDir: $nvimDir | nvimData: $nvimData | nvimCache: $nvimCache" 
  #
  $nvCustom = "${cfg}\{{configPath}}\main"
  $nvCustomData = "${dataDir}\{{configPath}}\main"
  $nvCustomCache = "${cacheDir}\{{configPath}}\main"
  #
  if (Get-Item -Path "${nvimDir}" | Select-Object -ExpandProperty LinkType) { rm $nvimDir } else {return}
  if (Get-Item -Path "${$nvimData}" | Select-Object -ExpandProperty LinkType) { rm $nvimData } else {return}
  if (Get-Item -Path "${$nvimCache}" | Select-Object -ExpandProperty LinkType) { rm $nvimCache } else {return}
  #
  if (test-path $nvCustom && !test-path "${nvimDir}") { mv "${nvCustom}" "${cfg}\\" }
  if (test-path $nvCustomData && !test-path "${nvimData}") { mv "${nvCustomData}" "${nvimData}\\" }
  if (test-path $nvCustomCache && !test-path "${nvimCache}") { mv "${nvCustomCache}" "${nvimCache}\\" }
  #
  rm -force -recurse "${cfg}\{{configPath}}"; 
  rm -force -recurse "${cfg}\ncm-rs"; 
  
# --| MacOS
_reset-macos:

# --| Reset To Default and Rebuild/Install NCM
reinstall: reset install
