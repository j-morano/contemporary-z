#!/usr/bin/env zsh

zc "$@"
zout=$(cat "/tmp/z_path")
zouts=("${(@s/|/)zout}")
if [ "${zouts[1]}" = "direct_cd" ]; then
    cd ${zouts[2]}
fi
