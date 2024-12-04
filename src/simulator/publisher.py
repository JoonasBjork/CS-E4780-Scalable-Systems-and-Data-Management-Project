# from queue import Queue
import redis
from datetime import datetime
import time
import requests
from multiprocessing import Pipe

from const import *

stream_names = {}

# Convert a string into a number based on character codes
def string_to_number(s):
    total = 0
    for char in s:
        total += ord(char)  # ord() gets the Unicode code point of a character
    return total

# Generate a stream name based on the string
def calculate_stream_name(s, worker_count):
    id_number = string_to_number(s) % worker_count + 1
    return f"s{id_number}"


def publisher_run_redis(queue) -> None:
    """ The publisher will emit event through redis stream"""
    try:
        stream_names = {}
        print("[PUBLISHER] Publisher started")

        iter = 0

        # Initialize the redis client parameter
        redis_client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT)
        pipeline = redis_client.pipeline()
        # stream_name = STREAM_NAME
        # redis_client.xtrim(stream_name, 0) # Cleanup the stream before starting up
        while True:
            items = []
            start = time.time()
            # while not queue.empty():
            while queue.poll():
                # item = queue.get()
                item = queue.recv()
                items.append(item)

            # if iter % 1000 == 0:
            #     print("[PUBLISHER] iter:", iter)
                # print("Item:", item)
                # print("Number of items in the queue:", queue.qsize())

            for item in items:
                # print("GOT ITEM FROM QUEUE", item)
                item_id = item["id"]
                if item_id not in stream_names:
                    # Small optimization to reduce calcultation
                    stream_names[item_id] = calculate_stream_name(item_id, WORKER_COUNT)
                
                stream_name = stream_names[item_id]
               
                iter += 1

                # Do three times :)
                pipeline.xadd(stream_name, item)
                pipeline.xadd(stream_name, item)
                pipeline.xadd(stream_name, item)
            pipeline.execute()      
            # if len(items) != 0:
            #     print(f"Pipeline execute with {len(items)} in {time.time() - start}")      
            #     print("[PUBLISHER] each records takes: ", time.time() - start)
    except KeyboardInterrupt:
        return
    
# def publisher_run_http(queue: Queue) -> None:
#     """ The publisher will emit event through http requests"""
#     # iter = 0
#     try:
#         print("[PUBLISHER] Publisher for http started")

#         # Initialize the redis client parameter
#         # TODO: Please change the parameter here if necessary
#         url = MANAGER_URL

#         while True:
#             # print("Publisher starting new iteration")
#             # if queue.not_empty: -- the queue.get blocks the thread anyways, this may also cause busy waiting
#             item = queue.get(block=True)
            
#             current_time = datetime.now().strftime('%H:%M:%S.%f')

#             # Convert the time field of data to seconds
#             data_time = item['time']

#             # Wait until the current time matches or exceeds the desired time
#             # while current_time < data_time:
#             #     time.sleep(0.1)  # Sleep for a second before checking again
#             #     current_time = datetime.now().strftime('%H:%M:%S.%f')
#                 # print(f"[PUBLISHER] next event at: {data_time}")
#                 # print(f"[PUBLISHER] current row: {item}")
            
#             item['time'] = item['time'].replace('.', ':') # To keep the original timestamp formatting, the fraction has to be separated with a colon

#             # Use content-type: application/x-www-form-urlencoded to avoid json
#             # print(f"TRYING TO POST TO {url}, with item {item}")
#             try:
#                 requests.post(url, data=item) 
#                 # print("publisher has sent:", iter)
#                 # iter += 1
#                 qsize = queue.qsize()
#                 if qsize > 500:
#                     print("The publisher is getting overwhelmed with too many items in queue: ", qsize)
#                 # print("Publisher queue size: ", queue.qsize())
#             except Exception as e:
#                 # Initially the deno manager may be slow at starting up. Therefore, we need to wait for a second before it can receive data. 
#                 print(f"Publisher tried posting to {url}, Got exception {e}")
#                 print("Publisher sleeping for 1 sec")
#                 time.sleep(1)
#     except KeyboardInterrupt:
#         return
    
