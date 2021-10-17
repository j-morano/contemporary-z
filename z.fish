set r (./target/release/z $argv)
if test $status -eq 1
    echo ERROR
else
    set rl (string split '#' $r)
    if test $rl[1] = "direct_cd"
        echo $rl[2]
        cd $rl[2]
    else if test $rl[1] = "clear"
        :
    else
        echo -e $rl[2]
        read -l -P 'Number: ' number
        set result (echo -e $rl[2] | awk (echo 'FNR == '$number' {print}'))
        set path (string split ' ' $result)
        ./target/release/z $path[2] &> /dev/null
        cd $path[2]
    end
end
