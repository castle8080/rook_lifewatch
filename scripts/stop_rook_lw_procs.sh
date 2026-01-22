#!/bin/sh

ps -eo pid,command | egrep 'rook_lw_(admin|daemon)' | while read pid cmd; do
    echo "Killing: $pid -> $cmd"
    kill $pid
done

p_count=$((ps -e | wc -l))
if [ $p_count -gt 0 ]; then
    echo "There are processes still running ($p_count)"
fi
