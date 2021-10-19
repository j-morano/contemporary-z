#!/usr/bin/env bash

zc "$@"
zout=$(cat "/tmp/z_path")
IFS='|'; zouts=($zout); unset IFS
if [ "${zouts[0]}" = "direct_cd" ]; then
    cd ${zouts[1]}
fi
