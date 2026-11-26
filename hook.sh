# Optional: also register the directories you reach with a plain `cd` (not
# only with `z`). Add it to ~/.bashrc or ~/.zshrc, after the `z` function.
if [[ -n "$ZSH_VERSION" ]]; then
    __cz_hook() { cz --add "$PWD" 2>/dev/null; }
    autoload -Uz add-zsh-hook
    add-zsh-hook chpwd __cz_hook
elif [[ -n "$BASH_VERSION" ]]; then
    __cz_prev_pwd="$PWD"
    __cz_hook() {
        if [[ "$PWD" != "$__cz_prev_pwd" ]]; then
            __cz_prev_pwd="$PWD"
            cz --add "$PWD" 2>/dev/null
        fi
    }
    PROMPT_COMMAND="__cz_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
