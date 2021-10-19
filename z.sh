#!/usr/bin/env fish

zc $argv
zout=$(cat "/tmp/z_path")
IFS=';'; zouts=($zout); unset IFS
if [ "${zouts[1]}" == "direct_cd" ]; then
    cd ${zouts[2]}
fi
