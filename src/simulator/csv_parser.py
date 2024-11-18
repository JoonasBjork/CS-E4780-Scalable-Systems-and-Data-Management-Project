import queue
import csv
import time
from datetime import datetime
from typing import Generator, Optional, Dict
import itertools


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


def parser_run(csv_file: str, queue: queue.Queue) -> None:
    try:
        byte_offset = 1000000
        print("[PARSER] Parser stated")

        shifted_time = 0  # Time corresponding to 00:00 for each cycle

        while True:
            byte_offset += 1000000
            # Advance the generator in larger steps so that initial processing is lighter
            generator = generator = read_from_offset(csv_file, byte_offset)

            # Get the first value that corresponds to the current time
            first_record = next(generator)
            if (first_record[TIME_OFFSET] and 
                first_record[PRICE_OFFSET] and 
                parse_time(first_record[TIME_OFFSET]) > datetime.now()):
                
                # print(f'[PARSER] found first event at {first_record[TIME_OFFSET]}')
                timestamp = parse_time(first_record[TIME_OFFSET])
                current_time = datetime.now()
                shifted_time = current_time - timestamp
                queue.put(csv_row_to_redis(first_record, current_time))
                break

        print("[PARSER] Starting to add items to queue")

        # iter = 0
        try:
            while True:
                # print("[PARSER] ON ITERATION", iter)
                # iter += 1
                record = next(generator)

                if record[TIME_OFFSET] == '':
                    queue.put(csv_row_to_redis(record, None))
                    continue
                
                record_timestamp = parse_time(record[TIME_OFFSET])

                while datetime.now() < record_timestamp:
                    # Wait until the current time is larger than the record's timestamp
                    time.sleep(0.1)

                timestamp = parse_time(record[TIME_OFFSET])
                queue.put(csv_row_to_redis(record, shifted_time + timestamp))
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