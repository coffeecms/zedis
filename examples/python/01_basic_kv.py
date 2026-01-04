import redis
import time

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 01 Basic Key-Value Operations ---")

# SET
print("Setting key 'user:1:name' to 'Alice'...")
r.set('user:1:name', 'Alice')

# GET
name = r.get('user:1:name')
print(f"Got value: {name}")

# EXPIRY
print("Setting key 'temp:otp' with 5s expiry...")
r.setex('temp:otp', 5, '123456')
ttl = r.ttl('temp:otp')
print(f"TTL: {ttl}")

# INCR
r.set('counter', 0)
new_val = r.incr('counter')
print(f"Counter incremented to: {new_val}")
