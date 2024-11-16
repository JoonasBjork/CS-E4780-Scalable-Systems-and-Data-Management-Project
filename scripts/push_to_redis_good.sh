#!/bin/bash
docker exec redis redis-cli XADD s1 \* id id1 sectype I last 11.105000 time 09:19:44:743 date 08-11-2021
