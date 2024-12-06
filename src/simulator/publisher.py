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

                pipeline.xadd(stream_name, item)
            pipeline.execute()      
            # if len(items) != 0:
            #     print(f"Pipeline execute with {len(items)} in {time.time() - start}")      
            #     print("[PUBLISHER] each records takes: ", time.time() - start)
    except KeyboardInterrupt:
        return
    
