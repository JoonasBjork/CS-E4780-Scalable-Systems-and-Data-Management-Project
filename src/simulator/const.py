CSV_FILE = '/data/debs2022-gc-trading-day-08-11-21.csv'
ID_OFFSET = 0
SEC_TYPE_OFFSET = 1
DATE_OFFSET = 2
PRICE_OFFSET = 21
TIME_OFFSET = 23

STREAM_NAME = 'ingress'

MANAGER_URL = 'http://manager:7777'

import os 
if os.environ['DEVLOCAL']:
    CSV_FILE = '../../data/debs2022-gc-trading-day-08-11-21.csv'
    MANAGER_URL = 'http://localhost:7777'
