#!/bin/bash
echo "STREAM ingress"
docker exec redis redis-cli XINFO STREAM ingress | awk '/length/{print; getline; print}'

# echo 
# echo
# echo
echo
echo "STREAM s1"
docker exec redis redis-cli XINFO STREAM s1 | awk '/length/{print; getline; print}'

# echo 
# echo
# echo
# echo
# echo "STREAM s2"
# docker exec redis redis-cli XINFO STREAM s2 | awk '/length/{print; getline; print}'
