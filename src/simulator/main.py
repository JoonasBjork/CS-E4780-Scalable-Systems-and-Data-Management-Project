# from queue import Queue
# import threading
from multiprocessing import Process, Pipe

from csv_parser import parser_run
from publisher import publisher_run_redis # publisher_run_http, 

from const import CSV_FILE

# task_queue = Queue()

receive_conn, produce_conn = Pipe()
# parser_thread = threading.Thread(target=parser_run, args=[CSV_FILE, task_queue])
# publisher_thread = threading.Thread(target=publisher_run_redis, args=[task_queue])

parser_thread = Process(target=parser_run, args=[CSV_FILE, produce_conn])
publisher_thread = Process(target=publisher_run_redis, args=[receive_conn])

# Start the threads
parser_thread.start()
publisher_thread.start()

# Wait for both threads to finish
parser_thread.join()
publisher_thread.join()

print("Main: All tasks are done.")