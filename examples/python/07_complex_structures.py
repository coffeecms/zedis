import redis

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 07 Complex Structures ---")

# LISTS
print("List Operations:")
r.rpush('mylist', 'A', 'B', 'C')
item = r.lpop('mylist')
print(f"Popped: {item}, Remaining: {r.lrange('mylist', 0, -1)}")

# HASHES
print("\nHash Operations:")
r.hset('myhash', mapping={'field1': 'value1', 'field2': 'value2'})
val = r.hget('myhash', 'field1')
print(f"Field1: {val}")

# SETS (Standard Redis Sets)
print("\nSet Operations:")
r.sadd('myset', 'apple', 'banana', 'apple') # 'apple' added once
members = r.smembers('myset')
print(f"Set members: {members}")
