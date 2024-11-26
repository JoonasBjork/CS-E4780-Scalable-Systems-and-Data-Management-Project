# from queue import Queue
# import threading
from multiprocessing import Process, Pipe, Queue

from csv_parser import parser_run
from publisher import publisher_run_redis # publisher_run_http, 

from const import CSV_FILE, PUBLISHER_COUNT


# parser_thread = threading.Thread(target=parser_run, args=[CSV_FILE, task_queue])
# publisher_thread = threading.Thread(target=publisher_run_redis, args=[task_queue])

receive_conn, produce_conn = Pipe()
parser_thread = Process(target=parser_run, args=[CSV_FILE, produce_conn])
publisher_thread = Process(target=publisher_run_redis, args=[receive_conn])

# task_queue = Queue()
# parser_thread = Process(target=parser_run, args=[CSV_FILE, task_queue])
# publisher_threads = [Process(target=publisher_run_redis, args=[task_queue]) for _ in range(PUBLISHER_COUNT)]
# print(f"{PUBLISHER_COUNT} publisher created")
# Start the threads
parser_thread.start()
# for publisher_thread in publisher_threads:
publisher_thread.start()

# Wait for both threads to finish
parser_thread.join()
# for publisher_thread in publisher_threads:
publisher_thread.join()

print("Main: All tasks are done.")