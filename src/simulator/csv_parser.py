# import queue
from multiprocessing import Pipe
import csv
import time
from datetime import datetime, timedelta
from typing import Generator, Optional, Dict
import itertools
import os
from itertools import islice
import pandas as pd

from const import *

today = datetime.today()

def parse_time(time_str: str) -> datetime:
    """ Convert mm:ss.1f format into total seconds, handling rollover logic. """
    try:
        timestamp = datetime.strptime(time_str, "%H:%M:%S.%f").time()
        timestamp = datetime.combine(today, timestamp)
        return timestamp
    except Exception as e:
        raise ValueError(f"Invalid time format: {time_str}") from e

def csv_row_to_redis(row: list[str], exact_time: Optional[datetime]) -> Dict[str, str]:
    ''' Read only required information from a csv row '''
    # print(f"CSV TO REDIS GOT ROW {row} and exact_time {exact_time}")
    if exact_time is None:
        return {
            'id': row[ID_OFFSET], 
            'sectype': row[SEC_TYPE_OFFSET],
            'last': row[PRICE_OFFSET],
            'time': '', 
            'date': row[DATE_OFFSET]
        }
     
    return {
        'id': row[ID_OFFSET], 
        'sectype': row[SEC_TYPE_OFFSET],
        'last': row[PRICE_OFFSET],
        'time': exact_time.strftime("%H:%M:%S.%f"), 
        'date': row[DATE_OFFSET]
    }



def read_batch_from_offset(filename: str, start_offset: int, chunk_size: int = 10000) -> Generator[str, None, None]:
    """Generator to read from a specific offset in the file and yield lines."""
    file = open(filename, 'r')
    file.seek(start_offset)
    if start_offset:
        while True:
            next_line = file.readline()
            if len(list(csv.reader([next_line]))[0]) == 40:
                break
    try:
        chunks = pd.read_csv(
            file,
            skip_blank_lines=False,
            chunksize=chunk_size,
            index_col=False        
        )

        for chunk in chunks:
            for _, line in chunk.iterrows():
                yield line.tolist()
    except FileNotFoundError as e:
        raise FileNotFoundError(f"""
            Error: File not found. Looking for file in folder /data/. Instead the file is at {filename}
            Please check your CSV_FILE in the .env file. Make sure it is in the format /data/csv_filename.csv
        """) from e

def get_next_n(iterator, n):
    return list(islice(iterator, n))

def parser_run(csv_file: str, queue) -> None:
    try:
        generator = read_batch_from_offset(csv_file, FILE_OFFSET_BYTES)
        first_record = next(generator)
        while pd.isna(first_record[TIME_OFFSET]):
            first_record = next(generator)
            
        time_shift = datetime.now() - parse_time(first_record[TIME_OFFSET])
     
        # print("[PARSER] Starting to add items to queue")
        # print("[PARSER] first_record", first_record)

        try:
            iter = 0
            while True:
                iter += 1
                record = next(generator)
                if pd.isna(record[TIME_OFFSET]):
                    queue.send(csv_row_to_redis(record, None))
                    continue

                # print("THIS IS THE TIME_OFFSET:", record[TIME_OFFSET])
                # record_timestamp = 
                sleep_duration: float = (parse_time(record[TIME_OFFSET]) - (datetime.now() - time_shift)).total_seconds()
                # print("[PARSER] found sleep duration")

                if sleep_duration > 0.1:
                    # print("[PARSER] sleeping for", sleep_duration, "On record with timestamp", parse_time(record[TIME_OFFSET]))
                    time.sleep(sleep_duration)

                # Add the records with the current time
                current_time: datetime = datetime.now()

                # record[TIME_OFFSET] = current_time.strftime("%H:%M:%S.%f")
                record[DATE_OFFSET] = current_time.strftime("%d-%m-%Y")

                if iter % 1000 == 0:
                    print("[PARSER] iter:", iter)
                    print("[PARSER] Parser lag:", -sleep_duration, "s")
                

                queue.send(csv_row_to_redis(record, current_time))
                
        except StopIteration:
            return
    except KeyboardInterrupt:
        return
    