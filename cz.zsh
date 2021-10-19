#!/usr/bin/env zsh

cz "$@"
zout=$(cat "/tmp/cz_path")
zouts=("${(@s/|/)zout}")
if [ "${zouts[1]}" = "direct_cd" ]; then
    cd ${zouts[2]}
fi
