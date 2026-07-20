#! /usr/bin/bash

set -ex

rustup default 1.90

# For correct formatting (see comments in rustfmt.toml)
rustup toolchain install nightly
rustup default stable
