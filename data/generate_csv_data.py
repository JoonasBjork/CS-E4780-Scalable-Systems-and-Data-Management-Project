import csv
import random
from datetime import datetime, timedelta

symbols = []
with open("unique_symbols.txt") as f:
    for line in f:
        symbols.append(line.strip())

# print(symbols)

output_length = 1_000_000

# Start generating data with random symbols

header = ["ID","SecType","Date","Time","Ask","Ask volume","Bid","Bid volume","Ask time","Days high ask","Close","Currency","Days high ask time","Days high","ISIN","Auction price","Days low ask","Days low","Days low ask time","Open","Nominal value","Last","Last volume","Trading time","Total volume","Mid price","Trading date","Profit","Current price","Related indices","Day high bid time", "Day low bid time","Open Time", "Last trade time", "Close Time", "Day high Time", "Day low Time","Bid time","Auction Time"] 

num_columns = len(header)



# The highest rate of messages was 1 million records in a 5 min period
# We will test the system with 2 million records in a 5 min period

# 2 million in a 5 min period is 6667 messages per second or one message per 0,00015 seconds. Therefore, we increment the timedate by 0,00015
messages_per_5_min = 1_000_000
messages_per_second = messages_per_5_min / 300
second_per_message = 1 / messages_per_second

print("messages_per_5_min:", messages_per_5_min)
print("messages_per_second:", messages_per_second)
print("second_per_message:", second_per_message)

time_increment = second_per_message # Seconds

time_increment_micro_second = timedelta(microseconds=time_increment * 1_000_000) # Microseconds
start_time = datetime.now().replace(hour=0, minute=0, second=0, microsecond=0)

sec_types = ["E", "I"]

output_filename = f"{output_length // 1_000_000}_million_at_{messages_per_5_min // 1_000_000}_million_messages_per_5min.csv"


with open(output_filename, mode='w', newline='', encoding='utf-8') as file:
    writer = csv.writer(file)
    
    # Write the header
    writer.writerow(header)

    for iteration in range(output_length):
        symbol = random.choice(symbols)
        last = random.uniform(0.1, 100_000)

        trading_time = start_time.strftime("%H:%M:%S.%f")[:-3]
        trading_date = start_time.strftime("%d-%m-%Y")

        row = [None] * num_columns

        row[0] = symbol
        row[1] = random.choice(sec_types)
        row[21] = last
        row[23] = trading_time
        row[26] = trading_date
        
        writer.writerow(row)

        if iteration % 100_000 == 0:
            print(f"On iteration {iteration}, timestamp: {start_time}")

        start_time = start_time + time_increment_micro_second
