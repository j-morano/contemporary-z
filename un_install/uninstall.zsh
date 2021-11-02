#!/usr/bin/env zsh

$HOME/.cargo/bin/cargo uninstall cz
rm -r ~/.local/share/cz/
awk '!/^([.] ~/[.]local/share/cz/cz[.]sh|# Alias for the Contemporary-z program)/' ~/.zshrc > ~/.zshrc

