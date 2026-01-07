import redis
import time

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 09 God Tier Probabilistic Data Structures ---")

# 1. HyperLogLog (Cardinality Estimation)
# Standard Redis API
print("\n[HyperLogLog]")
r.execute_command('PFADD', 'hll_users', 'user1', 'user2', 'user3')
r.execute_command('PFADD', 'hll_users', 'user2', 'user4') # user2 duplicate
count = r.execute_command('PFCOUNT', 'hll_users')
print(f"Estimated Unique Users: {count}") # Should be 4

# 2. Cuckoo Filter (Better Bloom Filter - Supports Deletion)
print("\n[Cuckoo Filter]")
r.execute_command('CF.ADD', 'cf_emails', 'alice@example.com')
exists = r.execute_command('CF.EXISTS', 'cf_emails', 'alice@example.com')
print(f"Alice Exists: {exists}")

# 3. Count-Min Sketch (Frequency Estimation)
print("\n[Count-Min Sketch]")
r.execute_command('CMS.INCRBY', 'cms_events', 'login', 5)
r.execute_command('CMS.INCRBY', 'cms_events', 'logout', 2)
r.execute_command('CMS.INCRBY', 'cms_events', 'login', 1)
login_count = r.execute_command('CMS.QUERY', 'cms_events', 'login')
print(f"Login Count (Approx): {login_count}") # Should be 6

# 4. Top-K (Heavy Hitters)
print("\n[Top-K]")
# Add items
r.execute_command('TOPK.ADD', 'top_products', 'iphone', 'samsung', 'iphone', 'pixel', 'iphone')
top_list = r.execute_command('TOPK.LIST', 'top_products') # Returns top items
print(f"Top Products: {top_list}")

# 5. t-digest (Percentiles/Quantiles)
print("\n[t-digest]")
# Add latencies
import random
for _ in range(100):
    r.execute_command('TDIGEST.ADD', 'latency_dist', random.uniform(10, 100))

# Get P99
p99 = r.execute_command('TDIGEST.QUANTILE', 'latency_dist', 0.99)
print(f"P99 Latency: {p99}")
