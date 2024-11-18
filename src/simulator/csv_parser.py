import queue
import csv
import time
from datetime import datetime
from typing import Generator

from const import *

today = datetime.today()

def parse_csv(csv_file: str) -> Generator[list[str], None, None]:
    """ Create a generator that read the csv line by line """
    with open(csv_file, mode='r') as file:
        reader = csv.reader(file)
        for _ in range(13):
            header = next(reader)
        for row in reader:
            yield row

def parse_time(time_str: str) -> float:
    """ Convert mm:ss.1f format into total seconds, handling rollover logic. """
    try:
        timestamp = datetime.strptime(time_str, "%H:%M:%S.%f").time()
        timestamp = datetime.combine(today, timestamp)
        return timestamp
    except ValueError as e:
        raise ValueError(f"Invalid time format: {time_str}") from e

def csv_row_to_redis(row: list[str], exact_time: int) -> dict[str, str]:
    ''' Read only required information from a csv row '''
    return {
        'id': row[ID_OFFSET], 
        'sectype': row[SEC_TYPE_OFFSET],
        'last': row[PRICE_OFFSET],
        'time': exact_time.strftime("%H:%M:%S.%f"), 
        'date': row[DATE_OFFSET]
    }

def parser_run(csv_file: str, queue: queue.Queue) -> None:
    try:
        # error_count, normal_count = 0, 0
        print("[PARSER] Parser stated")
        generator = parse_csv(csv_file)

        shifted_time = 0  # Time corresponding to 00:00 for each cycle

        while True:
            # Get the first valid row to initial the clock
            first_record = next(generator)
            if (first_record[TIME_OFFSET] and 
                first_record[PRICE_OFFSET] and 
                first_record[TIME_OFFSET] != '00:00:00.000'):
                
                print(f'[PARSER] found first event at {first_record[TIME_OFFSET]}')
                timestamp = parse_time(first_record[TIME_OFFSET])
                current_time = datetime.now()
                shifted_time = current_time - timestamp
                queue.put(csv_row_to_redis(first_record, current_time))
                break

        try:    
            while True:
                record = next(generator)
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