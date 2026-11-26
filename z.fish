function z
    # cz writes the directory to cd to in a private temporary file
    set -l zfile (mktemp); or return
    env CZ_PATH_FILE=$zfile cz $argv
    set -l zout (cat $zfile)
    rm -f $zfile
    if test -n "$zout"
        cd -- "$zout"
    end
end
