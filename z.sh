z() {
    local zfile zout
    zfile="${XDG_RUNTIME_DIR:+$XDG_RUNTIME_DIR/cz_path}"
    zfile="${zfile:-/tmp/cz_path_$USER}"
    cz "$@"
    zout=$(cat "$zfile" 2>/dev/null)
    if [[ -n "$zout" ]]; then
        cd -- "$zout"
    fi
}
