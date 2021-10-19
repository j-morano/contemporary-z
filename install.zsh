#!/usr/bin/env bash

cargo install --path cz
mkdir -p ~/.local/share/cz/
cp cz.zsh ~/.local/share/cz/
# If the alias has not already been added, add it
if ! grep -q "Contemporary-z" ~/.zshrc; then
    printf "\n# Alias for the Contemporary-z program\n" >> ~/.zshrc
    printf "alias z='. ~/.local/share/cz/cz.zsh'\n" >> ~/.zshrc
fi
