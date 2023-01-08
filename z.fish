function z
    cz $argv
    set zout (cat "/tmp/cz_path")
    if test -n "$zout"
        cd "$zout"
    end
end
