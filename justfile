#!/usr/bin/env -S just --justfile

set windows-shell := ["pwsh.exe","-NoLogo", "-noprofile", "-c"]
set shell := ["pwsh", "-c"]
set positional-arguments

myDir := `echo $HOME`

lint:
    cargo clippy

test:
    cargo test

debug:
    cargo build

release : test
    cargo build --release

run *args: debug
    rm ./target/debug/ncm || true
    mv -f ./target/debug/ncm-rs ./target/debug/ncm || true
    ./target/debug/ncm {{args}} || true

build : release
    rm ./target/release/ncm || true
    mv -f ./target/release/ncm-rs ./target/release/ncm || true

install: build
    cp ./target/release/ncm $HOME/.local/bin

# --| Manual Test Cleanup/Revert Changes --------
reset:
    test -L ~/.config/nvim && rm ~/.config/nvim || true
    mv ~/.config/nvim_configs/nvim ~/.config/ || true
    rm -rf ~/.config/nvim_configs || true
    rm -rf ~/.config/ncm-rs || true

# --| Reset To Default and Rebuild/Install NCM
reinstall: reset install

install-win:
    rm ./target/release/ncm.exe -ErrorAction SilentlyContinue
    "cargo test"
    "cargo build --release"
    mv -f ./target/release/ncm-rs ./target/release/ncm || true
    cp ./target/release/ncm.exe $HOME/.local/bin

