#!/bin/bash
docker exec -it $(docker ps | grep "timescale" | awk '{print $1}') psql --username username -d database
