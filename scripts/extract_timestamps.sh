#!/bin/bash

# Check if input file is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input_csv_file>"
    exit 1
fi

# Input file
input_file="$1"

# Check if the input file exists
if [ ! -f "$input_file" ]; then
    echo "Error: File '$input_file' not found!"
    exit 1
fi

# Generate output file name by appending '-timestamps' before the file extension
output_file="${input_file%.csv}-timestamps.csv"

# Extract timestamps and write to the output file
grep -v '^#' "$input_file" | cut -d',' -f24 > "$output_file"

echo "Timestamps extracted to '$output_file'"