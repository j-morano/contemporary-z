#!/usr/bin/env bash

$HOME/.cargo/bin/cargo install --path cz
mkdir -p ~/.local/share/cz/
cp cz.sh ~/.local/share/cz/
# If the alias has not already been added, add it
if ! grep -q "Contemporary-z" ~/.bashrc; then
    printf "\n# Alias for the Contemporary-z program\n" >> ~/.bashrc
    printf "alias z='. ~/.local/share/cz/cz.sh'\n" >> ~/.bashrc
fi
