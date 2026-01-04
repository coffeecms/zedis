import redis
import json

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 03 JSON Document Store ---")

user_data = {
    "name": "Bob",
    "age": 30,
    "skills": ["python", "rust", "docker"],
    "address": {
        "city": "New York",
        "zip": "10001"
    }
}

# JSON.SET
print("Storing JSON document...")
r.execute_command('JSON.SET', 'user:101', json.dumps(user_data))

# JSON.GET (Full)
print("Retrieving full document...")
full_doc = r.execute_command('JSON.GET', 'user:101', '.')
print(f"Full Doc: {full_doc}")

# JSON.GET (Path) - Note: In a real RedisJSON implementation you can query paths. 
# Zedis JSON.GET currently returns the full stored string if path support implies client-side parsing or simple string retrieval.
# Refer to Zedis documentation for advanced path support.
