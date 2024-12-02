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
# import pyarrow.csv as pacsv
import threading
import queue

from const import *

today = datetime.today()

def parse_time(time_str: str) -> datetime:
    """ Convert mm:ss.1f format into total seconds, handling rollover logic. """
    try:
        timestamp = datetime.strptime(time_str, "%H:%M:%S.%f").time()
        timestamp = datetime.combine(today, timestamp)
        return timestamp
    except ValueError as e:
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
        print(file.readline())
    chunks = pd.read_csv(
        file,
        skip_blank_lines=False,
        chunksize=chunk_size
    )
    for chunk in chunks:
        yield chunk.iterrows()


def read_csv_task(data_queue: queue.Queue, event: threading.Event, generator: Generator):
    while True:
        print("Waiting in read_csv_task")
        
        wait_start = time.time()
        event.wait()
        wait_end = time.time()
        print("Read_csv waited for:", wait_end - wait_start)

        event.clear()

        records_iterator = next(generator)
        data_queue.put(records_iterator)



def parser_run(csv_file: str, pipe) -> None:
    try:
        # Start the file from 15:00
        generator = read_batch_from_offset(csv_file, FILE_OFFSET_BYTES)
        
        # print("FOUND FIRST RECORD:", first_record)

        good_record_found = False # Needed for exiting the outer loop
        while not good_record_found:
            records = next(generator)
            for _, record in records:
                record = record.tolist()
                if record[TIME_OFFSET] != '':
                    time_shift = datetime.now() - parse_time(record[TIME_OFFSET])
                    print("Found first working record", record)
                    good_record_found = True # Make sure to exit the outer loop
                    break

        data_queue: queue.Queue = queue.Queue()
        csv_event = threading.Event()

        csv_read_thread = threading.Thread(target=read_csv_task, args=(data_queue, csv_event, generator,))
        csv_read_thread.start()

        print("[PARSER] Starting to add items to queue")

        # We can still use the first batch of records even though it wasn't 
        try:
            iter = 0
            while True:
                csv_event.set()
                for _, record in records:
                    record = record.tolist()

                    iter += 1
                    if pd.isna(record[TIME_OFFSET]):
                        pipe.send(csv_row_to_redis(record, None))
                        continue

                    sleep_duration: float = (parse_time(record[TIME_OFFSET]) - (datetime.now() - time_shift)).total_seconds()
                    # print("[PARSER] found sleep duration")

                    if sleep_duration > 0.1:
                        # print("[PARSER] sleeping for", sleep_duration, "On record with timestamp", parse_time(record[TIME_OFFSET]))
                        time.sleep(sleep_duration)

                    # Add the records with the current time
                    current_time: datetime = datetime.now()

                    record[DATE_OFFSET] = current_time.strftime("%d-%m-%Y")

                    if iter % 1000 == 0:
                        print("[PARSER] iter:", iter)
                        print("[PARSER] Parser lag:", -sleep_duration, "s")
                    

                    pipe.send(csv_row_to_redis(record, current_time))
                # out_start = time.time()
                wait_start = time.time()
                records = data_queue.get()
                wait_end = time.time()
                print("Main threadWaited for", wait_end - wait_start)
                # out_end = time.time()
                # print("Generator call took:", out_end - out_start, "Got len(records):", len(records))
        except StopIteration:
            return
    except KeyboardInterrupt:
        return
    finally:
        if csv_read_thread:
            csv_read_thread.join()
    

# def parse_csv(csv_file: str) -> Generator[list[str], None, None]:
#     """ Create a generator that read the csv line by line """
#     with open(csv_file, mode='r') as file:
#         reader = csv.reader(file)
#         for _ in range(13):
#             header = next(reader)
#         for row in reader:
#             yield row

# def read_from_offset(filename: str, start_offset: int, buffer_size: int = 128) -> Generator[str, None, None]:
#     """Generator to read from a specific offset in the file and yield lines."""
#     with open(filename, 'r') as file:
#         # Seek to the starting offset
#         file.seek(start_offset)

#         # Temporary buffer to hold chunks of file data
#         buffer = ''

#         reader = csv.reader(iter(file))
        
#         while True:
#             # Read a chunk from the file
#             chunk = file.read(buffer_size)
            
#             if not chunk:
#                 break  # End of file reached
            
#             # Add the chunk to the buffer
#             buffer += chunk

#             # Process the buffer, line by line
#             while '\n' in buffer:
#                 # line, buffer = buffer.split('\n', 1)  # Split at the first newline

#                 row = next(reader)
#                 yield row  # Yield the line, stripped of any extra whitespace

#         # Yield any remaining data in the buffer that wasn't followed by a newline
#         if buffer:
#             row = next(reader)
#             yield row



# def seek_to_next_line(file):
#     """Move the file pointer to the start of the next line."""
#     while True:
#         char = file.read(1)
#         if char == b'\n' or char == b'':
#             break


# def read_batch_with_pyarrow(filename: str, start_offset: int, chunk_size: int = 10000) -> Generator[list, None, None]:
#     with open(filename, 'rb') as f:
#         print("FIRST SEEK")
#         f.seek(start_offset)
#         print("SEEKING TO NEXT LINE")
#         seek_to_next_line(f)
#         print("READING CSV") 
#         reader = pacsv.read_csv( # This line doesn't work as it pulls the entire file into memory :(
#             f,
#             read_options=pacsv.ReadOptions(block_size=chunk_size)
#         )
#         print("YIELDING CSV")
#         for batch in reader.to_batches():
#             yield batch.to_pandas().values.tolist() # Return the full batch at once to reduce function calls