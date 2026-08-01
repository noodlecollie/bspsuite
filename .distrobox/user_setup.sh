#! /usr/bin/bash

set -ex

# Copy these where they need to go. Using a symbolic link has caused me to nuke
# my SSH keys before now when something's gone wrong (because you can't set
# permissions specifically on the link itself), and trying to mount these
# as volumes just doesn't work for some reason (files are 0 bytes and dirs are
# empty). There might be a better way, but I'm out of ideas.
cp -f "/home/$USER/.gitconfig" "$HOME/.gitconfig" && chmod 444 "$HOME/.gitconfig"
mkdir -p "$HOME/.ssh" && cp -rf /home/$USER/.ssh/id_* "$HOME/.ssh/" && chmod 755 "$HOME/.ssh/"

rustup show active-toolchain

if [ $? -ne 0]; then
	# For correct formatting (see comments in rustfmt.toml)
	rustup toolchain install nightly
	rustup default stable
fi
