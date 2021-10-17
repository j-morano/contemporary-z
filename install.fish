#!/usr/bin/env fish

cargo install --path zc
mkdir -p ~/.local/share/z/
cp z.fish ~/.local/share/z/
alias z ". ~/.local/share/z/z.fish"
funcsave z
