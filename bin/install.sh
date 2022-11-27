#!/bin/sh

set -e

$HOME/.cargo/bin/cargo install --path cz
mkdir -p $HOME/.local/share/cz/
cp cz.sh $HOME/.local/share/cz/
# If the alias has not already been added, add it
if ! grep -Eq "alias z\s*=\s*.*" $HOME/.bashrc; then
    printf "\nalias z='. $HOME/.local/share/cz/cz.sh'\n" >> $HOME/.bashrc
fi
