
z() {
    cz "$@"
    zout=$(cat "/tmp/cz_path")
    if [[ -n "$zout" ]]; then
        cd "$zout"
    fi
}
