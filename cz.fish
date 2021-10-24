#!/usr/bin/env fish

$HOME/.cargo/bin/cz $argv
set zout (cat "/tmp/cz_path")
set zouts (string split '|' $zout)
if test $zouts[1] = "command"
    eval $zouts[2]
end
