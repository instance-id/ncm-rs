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
dataPath := "nvim-ncm-data"
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
  /home/mosthated/_dev/languages/pwsh/file_sync/code_sync.ps1 /mnt/x/GitHub/instance-id/rust/ncm-rs
  echo "Reset Complete! {{configPath}}"
# @formatter:on

_reset-windows:
  just _refresh-code
  just _run-reset
  
_refresh-code:
  & C:\files\scripts\sync_code.ps1 Z:\code\rust\ncm-rs

# @formatter:off
# --| Windows
_run-reset:
  #!{{shebang}}
  try {
    $useLocal = if($null -ne $env:USE_LOCAL_PATH) { echo "Using Local Path"; [bool]::Parse($env:USE_LOCAL_PATH);  } else { $false }
    $cfg      = if(($null -ne $env:XDG_CONFIG_HOME) -and !($useLocal)) { $env:XDG_CONFIG_HOME } else { $env:LOCALAPPDATA }
    $dataDir  = if(($null -ne $env:XDG_DATA_HOME)   -and !($useLocal)) { $env:XDG_DATA_HOME   } else { $env:LOCALAPPDATA }
    $cacheDir = if(($null -ne $env:XDG_CACHE_HOME)  -and !($useLocal)) { $env:XDG_CACHE_HOME  } else { $env:LOCALAPPDATA }
    #
    $nvimDir = "${cfg}/nvim"
    $nvimData = "${dataDir}/nvim-data"
    $nvimCache = "${cacheDir}/nvim"
    echo "nvimDir: $nvimDir | nvimData: $nvimData | nvimCache: $nvimCache" 
    #
    $nvCustomDataDir = if ($useLocal) { "nvim-ncm-data" } else { "nvim-ncm" }
    $nvCustom = "${cfg}/nvim-ncm/main"
    $nvCustomData = "${dataDir}/${nvCustomDataDir}/main"
    $nvCustomCache = "${cacheDir}/${nvCustomDataDir}/main"
    echo "nvCustom: $nvCustom | nvCustomData: $nvCustomData | nvCustomCache: $nvCustomCache"    
    # 
    if ((test-path "${nvimDir}")   -and (Get-Item -Path "${nvimDir}"   | Select-Object -ExpandProperty LinkType)) { rm "${nvimDir}"   }
    if ((test-path "${nvimData}")  -and (Get-Item -Path "${nvimData}"  | Select-Object -ExpandProperty LinkType)) { rm "${nvimData}"  }
    if ((test-path "${nvimCache}") -and (Get-Item -Path "${nvimCache}" | Select-Object -ExpandProperty LinkType)) { rm "${nvimCache}" }
    #
    function renameDir([string]$dir,[string]$newName) {
      $newPath = $dir.Split("/")[-1]
      $newDir = "${dir}/$newName"
      if (!(test-path $newPath)){
        rename-item -path $dir -newname $newName
      }
    }
    # @formatter:on 
    if ((test-path "${nvCustom}") -and (!(test-path "${nvimDir}"))) { 
        renameDir "${nvCustom}" 'nvim' 
        mv "${cfg}/nvim-ncm/nvim" "${cfg}"
        echo "Moved: ${nvCustom} to ${cfg}" 
    }
    #
    if ((test-path "${nvCustomData}") -and (!(test-path "${nvimData}"))) {
        renameDir "${nvCustomData}"  'nvim-data'
        mv "${dataDir}/${nvCustomDataDir}/nvim-data"  "${dataDir}"
        echo "Moved: ${nvCustomData} to ${nvimData}" 
    }
    #
    if ((test-path "${nvCustomCache}") -and (!(test-path "${nvimCache}"))) { 
        mv "${nvCustomCache}" "${nvimCache}" ; 
        renameDir  ; 
        echo "Moved: ${nvCustomCache} to ${nvimCache}"
    }
    # @formatter:off
    if (test-path "${cfg}/nvim-ncm") { rm "${cfg}/nvim-ncm" -force -recurse; }
    if (test-path "${cfg}/ncm-rs")   { rm "${cfg}/ncm-rs"   -force -recurse; } 
  } catch { echo $_.Exception.Message; $_ > error.log }
  
# @formatter:on 

# --| MacOS
_reset-macos:

# --| Reset To Default and Rebuild/Install NCM
reinstall: reset install
