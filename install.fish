#!/usr/bin/env fish

cargo install --path cz
mkdir -p ~/.local/share/cz/
cp cz.fish ~/.local/share/cz/
alias z ". ~/.local/share/cz/cz.fish"
funcsave z
