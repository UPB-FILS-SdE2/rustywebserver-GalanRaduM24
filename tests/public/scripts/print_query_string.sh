#!/bin/bash

echo "Content-Type: text/plain"
echo
for variable in $(printenv | cut -d '=' -f 1)
do    
    if echo $variable | grep '^Query_' &> /dev/null 
    then
        echo "$variable=${!variable}"
    fi
done
