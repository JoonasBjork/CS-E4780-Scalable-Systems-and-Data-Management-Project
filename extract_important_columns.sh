#!/bin/bash

# Shell command to take only the important columns from the original csv file. 
grep -v '^#' debs2022-gc-trading-day-08-11-21.csv | cut -d',' -f1,2,22,24,27 > debs2022-gc-trading-day-08-11-21-clean.csv