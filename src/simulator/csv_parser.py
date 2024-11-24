# import queue
from multiprocessing import Pipe
import csv
import time
from datetime import datetime, timedelta
from typing import Generator, Optional, Dict
import itertools
import os


from const import *

today = datetime.today()

# def parse_csv(csv_file: str) -> Generator[list[str], None, None]:
#     """ Create a generator that read the csv line by line """
#     with open(csv_file, mode='r') as file:
#         reader = csv.reader(file)
#         for _ in range(13):
#             header = next(reader)
#         for row in reader:
#             yield row

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

def read_from_offset(filename: str, start_offset: int, buffer_size: int = 128) -> Generator[str, None, None]:
    """Generator to read from a specific offset in the file and yield lines."""
    with open(filename, 'r') as file:
        # Seek to the starting offset
        file.seek(start_offset)

        # Temporary buffer to hold chunks of file data
        buffer = ''

        reader = csv.reader(iter(file))
        
        while True:
            # Read a chunk from the file
            chunk = file.read(buffer_size)
            
            if not chunk:
                break  # End of file reached
            
            # Add the chunk to the buffer
            buffer += chunk

            # Process the buffer, line by line
            while '\n' in buffer:
                line, buffer = buffer.split('\n', 1)  # Split at the first newline

                row = next(reader)
                row = next(reader)
                yield row  # Yield the line, stripped of any extra whitespace

        # Yield any remaining data in the buffer that wasn't followed by a newline
        if buffer:
            row = next(reader)
            row = next(reader)
            yield row


def parser_run(csv_file: str, queue) -> None:
    try:

        generator = read_from_offset(csv_file, 100010000)
        first_record = next(generator)
        while first_record[TIME_OFFSET] == '':
            first_record = next(generator)
            # print("First record:", first_record)

        time_shift = datetime.now() - parse_time(first_record[TIME_OFFSET])


        print("[PARSER] Starting to add items to queue")
        # print("[PARSER] Current time:", datetime.now() - time_shift, "Time of the first_record", first_record)

        try:
            iter = 0
            while True:
                # iter += 1
                record = next(generator)

                if record[TIME_OFFSET] == '':
                    queue.send(csv_row_to_redis(record, None))
                    continue

                # record_timestamp = 
                sleep_duration: float = (parse_time(record[TIME_OFFSET]) - (datetime.now() - time_shift)).total_seconds()
                # print("[PARSER] found sleep duration")

                if sleep_duration > 0.1:
                    # print("[PARSER] sleeping for", sleep_duration, "On record with timestamp", parse_time(record[TIME_OFFSET]))
                    time.sleep(sleep_duration)
                # while datetime.now() - time_shift < record_timestamp:
                #     # Wait until the current time is larger than the record's timestamp
                #     print("[PARSER] In infinite loop", datetime.now() - time_shift, record_timestamp)
                #     time.sleep(0.1)
                # print("[Parser] out of infinite loop")

                timestamp = parse_time(record[TIME_OFFSET])
                
                # if iter % 100 == 0:
                #     print("[PARSER] iter:", iter)

                queue.send(csv_row_to_redis(record, timestamp))
                # if record[TIME_OFFSET] and record[PRICE_OFFSET]:
                #     normal_count += 1
                # else:
                #     error_count += 1
                # if (normal_count + error_count) % 1000000 == 0:
                #     print(f'Error percentage: {error_count / (normal_count + error_count)}')
        except StopIteration:
            return
    except KeyboardInterrupt:
        return