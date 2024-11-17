from queue import Queue
import redis
from datetime import datetime
import time

from const import *

def publisher_run(queue: Queue) -> None:
    try:
        print("[PUBLISHER] Publisher started")

        # Initialize the redis client parameter
        # TODO: Please change the parameter here if necessary
        redis_client = redis.Redis(host='localhost', port=9000, db=0)
        stream_name = STREAM_NAME
        redis_client.xtrim(stream_name, 0) # Cleanup the stream before starting up

        while True:
            if queue.not_empty:
                item = queue.get()
                
                current_time = datetime.now().strftime('%H:%M:%S.%f')

                # Convert the time field of data to seconds
                data_time = item['time']

                # Wait until the current time matches or exceeds the desired time
                while current_time < data_time:
                    time.sleep(0.1)  # Sleep for a second before checking again
                    current_time = datetime.now().strftime('%H:%M:%S.%f')
                    # print(f"[PUBLISHER] next event at: {data_time}")
                    # print(f"[PUBLISHER] current row: {item}")
                    

                redis_client.xadd(stream_name, item)
    except KeyboardInterrupt:
        return