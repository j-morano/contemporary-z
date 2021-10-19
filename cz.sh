#!/usr/bin/env bash

cz "$@"
zout=$(cat "/tmp/cz_path")
IFS='|'; zouts=($zout); unset IFS
if [ "${zouts[0]}" = "direct_cd" ]; then
    cd ${zouts[1]}
fi
