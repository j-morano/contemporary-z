# Optional: also register the directories you reach with a plain `cd` (not
# only with `z`). Save it as ~/.config/fish/conf.d/cz_hook.fish
function __cz_hook --on-variable PWD
    cz --add "$PWD" 2>/dev/null
end
