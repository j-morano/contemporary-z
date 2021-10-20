#!/usr/bin/env fish

cz $argv
set zout (cat "/tmp/cz_path")
set zouts (string split '|' $zout)
if test $zouts[1] = "direct_cd"
    cd $zouts[2]
end