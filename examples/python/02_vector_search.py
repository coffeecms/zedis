import redis

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 02 AI Vector Search (Auto-Embedding) ---")

# 1. Ingest Data (VADD.TEXT)
# Zedis automatically converts these strings to vectors using internal embeddings
docs = [
    ("doc:1", "The quick brown fox jumps over the lazy dog"),
    ("doc:2", "A fast russet wolf leaps over the sleepy canine"), # Semantically similar to doc:1
    ("doc:3", "Python programming is versatile and powerful"),
    ("doc:4", "Rust offers memory safety without garbage collection"),
]

print("Ingesting documents...")
for key, content in docs:
    r.execute_command('VADD.TEXT', key, content)
    print(f"Added {key}")

# 2. Semantic Search (VSEARCH.TEXT)
query = "rapid animal jumping"
print(f"\nSearching for: '{query}'")

# Search for top 2 matches
results = r.execute_command('VSEARCH.TEXT', 'doc', query, 2)
print("Results:", results)

# Expected: doc:2 and doc:1 (due to semantic similarity)
