#!/usr/bin/env bash

# clean metis process and data 
docker compose -f compose-mala.yaml down
docker compose down

rm -fr ./nodes
rm -fr ./rethdata
