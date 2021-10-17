#!/usr/bin/env fish

# Execute z core
set r (zc $argv)
if test $status -eq 1
    echo 'Error: '$r
else
    set rl (string split '#' $r)
    if test $rl[1] = "direct_cd"
        cd $rl[2]
    else if test $rl[1] = "clear"
        echo 'Cleared database'
    else
        if test $rl[2] -gt 0
            echo -e $rl[3]
            read -l -P 'Number: ' number
            if string match -qr '^[0-9]+$' $number
                if test $number -le $rl[2]
                    set result (echo -e $rl[3] | awk (echo 'FNR == '$number' {print}'))
                    set path (string split ' ' $result)
                    zc $path[2] &> /dev/null
                    cd $path[2]
                else
                    echo 'Error: No folder with number '$number
                end
            else
                echo 'Exit: No number selected'
            end
        else
            echo 'Exit: No results'
        end
    end
end
