#!/usr/bin/env fish

zc $argv
set zout (cat "/tmp/z_path")
set zouts (string split '|' $zout)
if test $zouts[1] = "direct_cd"
    cd $zouts[2]
end
