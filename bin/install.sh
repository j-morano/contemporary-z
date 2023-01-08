#!/bin/sh

set -e

$HOME/.cargo/bin/cargo install --path cz
mkdir -p $HOME/.local/share/contemporary-z/
cp cz.sh $HOME/.local/share/contemporary-z/
# If the alias has not already been added, add it
if ! grep -Eq "alias z\s*=\s*.*" $HOME/.bashrc; then
    printf "\nalias z='. $HOME/.local/share/contemporary-z/cz.sh'\n" >> $HOME/.bashrc
fi
