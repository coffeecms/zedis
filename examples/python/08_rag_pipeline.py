import redis

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 08 Advanced: Basic RAG Pipeline Mock ---")

# Scenario: A knowledge base for a tech support bot.

kb_data = [
    ("kb:1", "To reset your password, go to settings and click 'Reset Password'."),
    ("kb:2", "If the device won't turn on, hold the power button for 10 seconds."),
    ("kb:3", "Error 500 means internal server error. Check the logs."),
    ("kb:4", "For billing inquiries, contact support@example.com."),
]

# 1. Indexing
print("Indexing Knowledge Base...")
for k, content in kb_data:
    r.execute_command('VADD.TEXT', k, content)

# 2. Retrieval (The 'R' in RAG)
user_query = "cloud I turn on my device?" # Intentionally typo'd (cloud vs could)
print(f"\nUser Query: {user_query}")

# Retrieve top 1 most relevant doc
context = r.execute_command('VSEARCH.TEXT', 'kb', user_query, 1)
print(f"Retrieved Context: {context}")

# 3. Generation (Mock)
# In a real app, you would pass 'context' + 'user_query' to an LLM like GPT-4.
print("\n[Mock LLM Response]")
if context and len(context) > 0:
    print(f"Based on the context '{context[0]}', here is the answer:")
    print("You should try holding the power button for 10 seconds.")
else:
    print("No relevant context found. Please check if:")
    print("  1. Zedis is running with BGE-M3 loaded successfully")
    print("  2. VADD.TEXT commands succeeded (no errors in logs)")
    print("  3. The search index contains documents")
