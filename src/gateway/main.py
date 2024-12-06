# from queue import Queue
# import threading
from multiprocessing import Process, Pipe, Queue

from csv_parser import parser_run
from publisher import publisher_run_redis # publisher_run_http, 

from const import CSV_FILE, PUBLISHER_COUNT


receive_conn, produce_conn = Pipe()
parser_thread = Process(target=parser_run, args=[CSV_FILE, produce_conn])
publisher_thread = Process(target=publisher_run_redis, args=[receive_conn])

# Start the threads
parser_thread.start()
# for publisher_thread in publisher_threads:
publisher_thread.start()

# Wait for both threads to finish
parser_thread.join()
# for publisher_thread in publisher_threads:
publisher_thread.join()

print("Main: All tasks are done.")