#!/bin/bash

# Shell command to take only timestamps from the original csv data
grep -v '^#' debs2022-gc-trading-day-08-11-21.csv | cut -d',' -f24 > debs2022-gc-trading-day-08-11-21-timestamps.csv
