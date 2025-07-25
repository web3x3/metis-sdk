#!/usr/bin/env bash

# start metis node as engine
docker compose up -d

# waiting for metis node completely running
sleep 5

# start malaketh-layered node as consensus
docker compose -f compose-mala.yaml up -d
