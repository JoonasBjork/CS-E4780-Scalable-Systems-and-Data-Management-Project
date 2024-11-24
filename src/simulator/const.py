CSV_FILE = '/data/debs2022-gc-trading-day-08-11-21.csv'
ID_OFFSET = 0
SEC_TYPE_OFFSET = 1
DATE_OFFSET = 2
PRICE_OFFSET = 21
TIME_OFFSET = 23

REDIS_HOST = 'redis-streams'
REDIS_PORT = 6379
STREAM_NAME = 'ingress'

MANAGER_URL = 'http://manager:7777'

import os 

WORKER_COUNT = os.environ.get("WORKER_COUNT")

if WORKER_COUNT is None:
    WORKER_COUNT = 1
else:
    try:
        WORKER_COUNT = int(WORKER_COUNT)
        print("Set WORKER_COUNT to:", WORKER_COUNT)
    except Exception as e:
        print("WORKER_COUNT caused error:", e)
        print("Setting WORKER_COUNT TO 1")
        WORKER_COUNT = 1

MESSAGE_MULTIPLIER = os.environ.get("MESSAGE_MULTIPLIER")

if MESSAGE_MULTIPLIER is None:
    MESSAGE_MULTIPLIER = 1
else:
    try:
        MESSAGE_MULTIPLIER = int(MESSAGE_MULTIPLIER)
        print("Set MESSAGE_MULTIPLIER to:", MESSAGE_MULTIPLIER)
    except Exception as e:
        print("MESSAGE_MULTIPLIER caused error:", e)
        print("Setting MESSAGE_MULTIPLIER TO 1")
        MESSAGE_MULTIPLIER = 1

devlocal = os.environ.get('DEVLOCAL')
if devlocal is not None and devlocal.lower() in ['1', 'true', 'yes']:
    CSV_FILE = '../../data/debs2022-gc-trading-day-08-11-21.csv'
    MANAGER_URL = 'http://localhost:7777'
    REDIS_PORT = 9000
    REDIS_HOST = 'localhost'