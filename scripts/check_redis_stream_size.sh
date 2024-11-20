#!/bin/bash
echo "STREAM ingress"
docker exec redis redis-cli XINFO STREAM ingress

echo 
echo
echo "STREAM s1"
docker exec redis redis-cli XINFO STREAM s1

echo 
echo
echo "STREAM s1"
docker exec redis redis-cli XINFO STREAM s2
