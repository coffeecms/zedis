import redis

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 04 Bloom Filter (Probabilistic) ---")

filter_key = "uniq_visitors"

# BF.ADD
print("Adding visitors...")
r.execute_command('BF.ADD', filter_key, "user_1")
r.execute_command('BF.ADD', filter_key, "user_2")
r.execute_command('BF.ADD', filter_key, "user_3")

# BF.EXISTS
print("\nChecking existence:")
users_to_check = ["user_1", "user_5", "user_2"]

for u in users_to_check:
    exists = r.execute_command('BF.EXISTS', filter_key, u)
    # Zedis returns 1 for True, 0 for False
    status = "Found" if exists == 1 else "Not Found"
    print(f"User {u}: {status}")
