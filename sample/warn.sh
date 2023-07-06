#!/bin/bash

# Environment variables passed by running templater
# echo $root_usage
# echo $memory_usage
# echo $swap_usage
# echo $load
# echo $cpu_cores

# Float -> Integers
load=${load%.*}
memory_usage=${memory_usage%.*}
swap_usage=${swap_usage%.*}

if (( root_usage > 90 )) ; the
    echo -e "\e[33m Warning: Disk usage of / is too high. \e[0m"
fi

if (( memory_usage >= 90 )); then
    echo -e "\e[33m Warning: Memory usage is too high \e[0m"
fi

if (( swap_usage >= 50 )); then
    echo -e "\e[33m Warning: Swap is used heaviliy \e[0m"
fi

threshold=$(( cpu_cores * 2 - cpu_cores / 2 ))

if (( $(echo "$load > $cpu_cores " | bc -l))); then
    echo -e "\e[33m Warning: System load is high \e[0m"
fi
