#!/usr/bin/env bash

cd ../lib/malaketh-layered
make clean
cd -

ps aux | grep "metis" | grep -v grep | awk '{print $2}' | xargs kill
ps aux | grep "mala" | grep -v grep | awk '{print $2}' | xargs kill
rm -fr ./test
