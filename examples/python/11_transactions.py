import redis

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 11 Transactions (MULTI/EXEC) ---")

key1 = "account:A"
key2 = "account:B"

r.set(key1, 100)
r.set(key2, 50)

print(f"Initial: A={r.get(key1)}, B={r.get(key2)}")

# Start Transaction
print("Starting Transaction...")
pipe = r.pipeline(transaction=True) # transaction=True uses MULTI/EXEC

pipe.decrby(key1, 20)
pipe.incrby(key2, 20)

# Execute
print("Executing...")
results = pipe.execute()

print(f"Results: {results}") # [80, 70]
print(f"Final: A={r.get(key1)}, B={r.get(key2)}")

# Discard Example
print("\n[DISCARD Test]")
pipe = r.pipeline(transaction=True)
pipe.set("temp_key", "should_not_exist")
print("Discarding transaction...")
# redis-py's pipeline wraps DISCARD logic in reset(), but strictly speaking we interact via commands if doing manually.
# Using raw commands to demonstrate:
try:
    r.execute_command("MULTI")
    r.execute_command("SET", "temp_key", "fail")
    r.execute_command("DISCARD")
    print("Transaction Discarded.")
except Exception as e:
    print(e)

print(f"temp_key exists? {r.exists('temp_key')}")
