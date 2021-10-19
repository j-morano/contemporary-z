#!/usr/bin/env bash

cargo install --path zc
mkdir -p ~/.local/share/z/
cp z.sh ~/.local/share/z/
# If the alias has not already been added, add it
if ! grep -q "Contemporary-z" ~/.bashrc; then
    printf "\n# Alias for the Contemporary-z program\n" >> ~/.bashrc
    printf "alias z='. ~/.local/share/z/z.sh'\n" >> ~/.bashrc
fi
