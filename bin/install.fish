#!/usr/bin/env fish

$HOME/.cargo/bin/cargo install --path cz
mkdir -p ~/.local/share/contemporary-z/
cp cz.fish ~/.local/share/contemporary-z/
alias z ". ~/.local/share/contemporary-z/cz.fish"
funcsave z
