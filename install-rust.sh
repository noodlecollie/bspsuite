#! /usr/bin/bash

# Helper script to install the relevant versions of Rust in the Distrobox container.
# This is separate from distrobox.ini because it needs to run as non-root,
# and I've spent ages trying to work that out to no avail.
set -ex
rustup default 1.90

# For correct formatting (see comments in rustfmt.toml)
rustup toolchain install nightly
rustup default stable
