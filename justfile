#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe"]

release:
  cargo build --release    

lint:
  cargo clippy



