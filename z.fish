function z
    cz $argv
    if test -n "$XDG_RUNTIME_DIR"
        set zfile "$XDG_RUNTIME_DIR/cz_path"
    else
        set zfile "/tmp/cz_path_$USER"
    end
    set zout (cat "$zfile" 2>/dev/null)
    if test -n "$zout"
        cd -- "$zout"
    end
end
