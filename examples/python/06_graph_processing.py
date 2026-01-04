import redis

# Connect to Zedis
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

print("--- 06 Graph Processing ---")

graph_key = "social_net"

# GRAPH.ADD (Key, Node, Neighbor)
# Creating a simple directed graph: Alice -> Bob -> Charlie
print("Building graph...")
r.execute_command('GRAPH.ADD', graph_key, "Alice", "Bob")
r.execute_command('GRAPH.ADD', graph_key, "Bob", "Charlie")
r.execute_command('GRAPH.ADD', graph_key, "Alice", "Dave")

# GRAPH.BFS (Key, StartNode, MaxDepth)
# Find all nodes reachable from Alice within 2 steps
print("\nPerforming BFS from Alice (Depth 2)...")
visited = r.execute_command('GRAPH.BFS', graph_key, "Alice", 2)
print(f"Reachable nodes: {visited}")
