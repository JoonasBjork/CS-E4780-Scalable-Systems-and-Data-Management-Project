from typing import Optional
import os 

def get_str_envvar(envvar_name: str, default_value: Optional[str] = None) -> str:
    # If no default value is given, throw an exception

    value = os.environ.get(envvar_name)
    if value is not None:
        print(f"Set environment variable {envvar_name} to {value}")
        return value

    if default_value is not None:
        print(f"Set environment variable {envvar_name} to {default_value}")
        return default_value
    
    raise RuntimeError(f"envvar {envvar_name} does not exist and no default value was given")

    
def get_int_envvar(envvar_name: str, default_value: Optional[int]) -> int:
    try:
        str_default_value = str(default_value) if default_value is not None else None
        str_envvar = get_str_envvar(envvar_name, str_default_value)
        return int(str_envvar)
    except ValueError:
        raise RuntimeError(
            f"envvar {envvar_name} could not be converted to an integer"
        )
    except Exception as e:
        raise RuntimeError(f"Unexpected error in get_int_envvar: {e}")

# CSV_FILE = '/data/debs2022-gc-trading-day-08-11-21.csv'
# FILE_OFFSET_BYTES = 3100000000
# When reading different files, use different offets
CSV_FILE = '/data/1_million_at_2_million_messages_per_5min.csv'
FILE_OFFSET_BYTES = 1 

ID_OFFSET = 0
SEC_TYPE_OFFSET = 1
PRICE_OFFSET = 21
TIME_OFFSET = 23
DATE_OFFSET = 26

WORKER_COUNT = get_int_envvar("WORKER_COUNT", 1)
REDIS_HOST = get_str_envvar("REDIS_HOST")
REDIS_PORT = get_str_envvar("REDIS_PORT")
PUBLISHER_COUNT = get_int_envvar("PUBLISHER_COUNT", 1)

devlocal = os.environ.get('DEVLOCAL')
if devlocal is not None and devlocal.lower() in ['1', 'true', 'yes']:
    CSV_FILE = '../../data/debs2022-gc-trading-day-08-11-21.csv'
    MANAGER_URL = 'http://localhost:7777'
    REDIS_PORT = 9000
    REDIS_HOST = 'localhost'