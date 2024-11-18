from queue import Queue
import threading

from csv_parser import parser_run
from publisher import publisher_run_http, publisher_run_redis

from const import CSV_FILE

task_queue = Queue()

parser_thread = threading.Thread(target=parser_run, args=[CSV_FILE, task_queue])
publisher_thread = threading.Thread(target=publisher_run_http, args=[task_queue])

# Start the threads
parser_thread.start()
publisher_thread.start()

# Wait for both threads to finish
parser_thread.join()
publisher_thread.join()

print("Main: All tasks are done.")