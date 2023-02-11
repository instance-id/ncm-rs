#!/usr/bin/env just --justfile

set positional-arguments

release:
  cargo build --release    
  
debug:
    cargo build

lint:
  cargo clippy

bin:
  cargo run --bin bin -- arg1

example:
  cargo run --example exname -- arg1
  
run args: debug 
    rm ./target/debug/ncm || true
    mv -f ./target/debug/ncm-rs ./target/debug/ncm || true 
    ./target/debug/ncm {{args}} 
