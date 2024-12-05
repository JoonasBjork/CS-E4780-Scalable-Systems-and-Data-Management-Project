#!/bin/bash
while true; do
    docker exec redis redis-cli DEL s1
done