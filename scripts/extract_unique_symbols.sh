#!/bin/bash
awk -F, '{print $1}' debs2022-gc-trading-day-08-11-21.csv | sort | uniq > unique_symbols.txt