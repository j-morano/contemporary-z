z() {
    local zfile zout
    # cz writes the directory to cd to in a private temporary file
    zfile=$(mktemp) || return
    CZ_PATH_FILE="$zfile" cz "$@"
    zout=$(cat "$zfile")
    rm -f "$zfile"
    if [[ -n "$zout" ]]; then
        cd -- "$zout"
    fi
}
