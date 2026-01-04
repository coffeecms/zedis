import redis
import time

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 05 Time Series ---")

ts_key = "sensor:temp:room1"

# TS.ADD (Key, Timestamp, Value)
start_time = int(time.time() * 1000)

print(f"Adding data points to {ts_key}...")
r.execute_command('TS.ADD', ts_key, start_time, 22.5)
r.execute_command('TS.ADD', ts_key, start_time + 1000, 23.0)
r.execute_command('TS.ADD', ts_key, start_time + 2000, 23.2)
r.execute_command('TS.ADD', ts_key, start_time + 3000, 22.8)

# TS.RANGE (Key, Start, End)
print("\nQuerying range:")
# Query from start_time to start_time + 5000
data = r.execute_command('TS.RANGE', ts_key, start_time, start_time + 5000)
print(data)
