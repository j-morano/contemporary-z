#!/usr/bin/env bash

~/.cargo/bin/cz "$@"
zout=$(cat "/tmp/cz_path")
IFS='|'; zouts=($zout); unset IFS
if [ "${zouts[0]}" = "command" ]; then
    eval ${zouts[1]}
fi
