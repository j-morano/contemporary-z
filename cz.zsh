#!/usr/bin/env zsh

~/.cargo/bin/cz "$@"
zout=$(cat "/tmp/cz_path")
zouts=("${(@s/|/)zout}")
if [ "${zouts[1]}" = "command" ]; then
    eval ${zouts[2]}
fi
