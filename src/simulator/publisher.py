from queue import Queue
import redis
from datetime import datetime
import time
import requests

from const import *

def publisher_run_redis(queue: Queue) -> None:
    """ The publisher will emit event through redis stream"""
    try:
        print("[PUBLISHER] Publisher started")

        # Initialize the redis client parameter
        # TODO: Please change the parameter here if necessary
        redis_client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT)
        stream_name = STREAM_NAME
        redis_client.xtrim(stream_name, 0) # Cleanup the stream before starting up
        try:
            redis_client.xgroup_create("ingress", "managers", "$", mkstream=True)
        except Exception as e:
            print(e)
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
    
def publisher_run_http(queue: Queue) -> None:
    """ The publisher will emit event through http requests"""
    iter = 0
    try:
        print("[PUBLISHER] Publisher for http started")

        # Initialize the redis client parameter
        # TODO: Please change the parameter here if necessary
        url = MANAGER_URL

        while True:
            # print("Publisher starting new iteration")
            # if queue.not_empty: -- the queue.get blocks the thread anyways, this may also cause busy waiting
            item = queue.get(block=True)
            
            current_time = datetime.now().strftime('%H:%M:%S.%f')

            # Convert the time field of data to seconds
            data_time = item['time']

            # Wait until the current time matches or exceeds the desired time
            # while current_time < data_time:
            #     time.sleep(0.1)  # Sleep for a second before checking again
            #     current_time = datetime.now().strftime('%H:%M:%S.%f')
                # print(f"[PUBLISHER] next event at: {data_time}")
                # print(f"[PUBLISHER] current row: {item}")
            
            item['time'] = item['time'].replace('.', ':') # To keep the original timestamp formatting, the fraction has to be separated with a colon

            # Use content-type: application/x-www-form-urlencoded to avoid json
            # print(f"TRYING TO POST TO {url}, with item {item}")
            try:
                requests.post(url, data=item) 
                # print("publisher has sent:", iter)
                # iter += 1
                qsize = queue.qsize()
                if qsize > 500:
                    print("The publisher is getting overwhelmed with too many items in queue: ", qsize)
                # print("Publisher queue size: ", queue.qsize())
            except Exception as e:
                # Initially the deno manager may be slow at starting up. Therefore, we need to wait for a second before it can receive data. 
                print(f"Publisher tried posting to {url}, Got exception {e}")
                print("Publisher sleeping for 1 sec")
                time.sleep(1)
    except KeyboardInterrupt:
        return
    
