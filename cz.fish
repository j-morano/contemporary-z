#!/usr/bin/env fish

$HOME/.cargo/bin/cz $argv
set zout (cat "/tmp/cz_path")
if test -n "$zout"
    cd "$zout"
end
