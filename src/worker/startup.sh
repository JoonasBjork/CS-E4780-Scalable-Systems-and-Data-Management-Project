#!/bin/bash
# A small hack to retrieve the index of the replica. 

# From here https://github.com/docker/compose/issues/9153
REPLICA_INDEX=$( v="$( nslookup "$( hostname -i )" | head -n 1 )"; v="${v##* = }"; v="${v%%.*}"; v="${v##*-}"; v="${v##*_}"; echo "$v" )

export REPLICA_INDEX

echo "Container ID is: $REPLICA_INDEX"

exec /usr/local/bin/worker