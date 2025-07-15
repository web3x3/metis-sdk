#!/usr/bin/env bash

# clean malaketh-layered process and data
cd ../lib/malaketh-layered
make clean
cd -

# clean metis process and data 
for pid in $(ps -ef | grep "mala" | grep -v grep | awk '{print $2}'); do
    kill -9 "$pid"
done

for pid in $(ps -ef | grep "metis" | grep -v grep | awk '{print $2}'); do
    kill -9 "$pid"
done

rm -fr ./test
