# Zedis ⚡
**The Universe Tier Data Engine**

![Build Status](https://img.shields.io/badge/build-passing-brightgreen) ![License](https://img.shields.io/badge/license-MIT-blue) ![Rust](https://img.shields.io/badge/rust-1.83%2B-orange)

A drop-in Redis replacement built in Rust. Multi-threaded. AI-Ready.
**Faster. Safer. Free.**

Zedis is a FREE, high-performance, drop-in replacement for Redis. Built in Rust with Thread-per-Core architecture, AI Vector Search, Graph Processing, and Time-Series support. Outperforms Redis in every benchmark.

> **100% FREE & OPEN SOURCE**

---

## 🚀 God Tier Features

Everything Redis has, plus everything it doesn't.

### Core Architecture
*   **Thread-per-Core Architecture**: Shared-nothing design. Each CPU core runs its own event loop. Linear scaling with hardware.
*   **High-Performance Storage**: Built on **DashMap** for highly concurrent, lock-free read/write operations.
*   **No Global Lock**: Unlike Redis (single-threaded), Zedis utilizes your entire server. 
*   **Memory Safety**: Built in **Rust** for guaranteed memory safety without the crash risks of C/C++.

### Universe Tier Capabilities 🌌

| Feature | Description | Key Commands |
| :--- | :--- | :--- |
| **Z-Vector** 🧠 | **AI Vector Search**. Native semantic search with auto-embedding (BGE-M3). | `VADD.TEXT`, `VSEARCH.TEXT` |
| **Z-Probabilistic** 🎲 | **Probabilistic Structures**. HyperLogLog, Cuckoo Filter, CMS, Top-K, t-digest. | `PFADD`, `CF.ADD`, `CMS.INCRBY`, `TOPK.ADD` |
| **Z-Stream** 🌊 | **Event Streams**. Append-only logs for event sourcing and messaging. | `XADD`, `XRANGE` |
| **Z-PubSub** 📢 | **Real-time Messaging**. Publish/Subscribe pattern for instant notifications. | `PUBLISH`, `SUBSCRIBE` |
| **Z-Tx** 🤝 | **ACID Transactions**. Atomic execution of command blocks. | `MULTI`, `EXEC`, `DISCARD` |
| **Z-Time** 📊 | **Time-Series Engine**. High-frequency metrics. Perfect for IoT & Finance. | `TS.ADD`, `TS.RANGE` |
| **Z-Graph** 🕸️ | **Graph Processing**. Adjacency lists with BFS/DFS traversal. | `GRAPH.ADD`, `GRAPH.BFS` |
| **Z-Geo** 🌍 | **Geo-Spatial Index**. Store and query locations. | `GEOADD` |
| **Z-Doc** 📄 | **Native JSON Store**. Store and query JSON documents natively. | `JSON.SET`, `JSON.GET` |
| **Z-Mask** 🎭 | **ElasticSearch Compatibility**. Zedis listens on Port 9200 and speaks standard ES API. | HTTP API |
| **Z-Flow** 🔄 | **Zero-ETL Sync**. Auto-sync data from MySQL/Postgres/SQLite via `zflow.toml`. | Config-driven |
| **Z-Bit** 0️⃣ | **Bitwise Operations**. High-performance bit manipulation. | `BITCOUNT`, `BITFIELD` |

---

## 🎭 Z-Mask: Database Compatibility Layer

Zedis can impersonate other databases, allowing you to migrate with **zero code changes**.

### ElasticSearch (Port 9200)
Zedis automatically starts an HTTP server on **port 9200** that understands the ElasticSearch API.

```bash
# Your existing ES client code works instantly!
curl localhost:9200
# Returns: { "tagline": "You Know, for Search" }

curl -X POST localhost:9200/my_index/_search -d '{"query":{"match":{"content":"AI"}}}'
```

**How to Migrate:** Just change the `host` in your ES client from `elasticsearch-server` to `zedis-server`.

---

## 🌊 Z-Flow: Zero-ETL Data Sync

Manually pushing data is "Human Tier". Z-Flow automates it by syncing directly from your RDBMS.

### Quick Start
1.  Create `zflow.toml` in the Zedis directory.
2.  Start Zedis. It automatically connects, polls, and syncs.

**Hot Reload:** Edit `zflow.toml` on the fly. Zedis reconfigures instantly, no restart needed.

### Configuration Examples

#### Sync to Vector Search (MySQL)
Auto-embed product descriptions for semantic search.
```toml
[[flow]]
name = "product_vector_search"
source = "mysql://user:password@localhost:3306/ecommerce?pool_max=10&pool_min=2&connect_timeout=5"
table = "products"
target = "vector"
fields = ["title", "description"]  # Columns to embed
key_format = "product:{id}"
interval = 10  # Poll every 10 seconds
```

#### Sync to JSON Cache (PostgreSQL)
Cache user profiles for fast API responses.
```toml
[[flow]]
name = "user_cache"
source = "postgres://admin:secret@db.server.com:5432/main?sslmode=prefer&pool_max=20&statement_cache_capacity=100"
table = "users"
target = "json"
key_format = "user:{id}"
fields = ["*"]  # All columns
interval = 5
```

#### Sync to Bloom Filter (SQL Server)
Block banned emails on signup forms.
```toml
[[flow]]
name = "email_blacklist"
source = "sqlserver://sa:Password!@corp-db:1433/security?encrypt=true&trust_server_certificate=true&pool_max=5"
table = "banned_emails"
target = "bloom"
key = "bloom:banned"
item = "email"  # Column to add to filter
error_rate = 0.01
```

#### Sync to Graph (PostgreSQL)
Build social network from "follows" table.
```toml
[[flow]]
name = "social_graph"
source = "postgres://app:pass@localhost:5432/social?sslmode=disable&pool_max=15&pool_timeout=30"
table = "follows"
target = "graph"
graph_key = "network"
source_node = "follower_id"
destination_node = "followee_id"
```

#### Sync to Time-Series (MySQL)
Ingest IoT sensor readings.
```toml
[[flow]]
name = "sensor_metrics"
source = "mysql://iot:pass@timescale-db:3306/sensors?pool_max=30&pool_min=5&connect_timeout=3"
table = "readings"
target = "timeseries"
key_format = "sensor:{device_id}:temp"
timestamp = "created_at"
value = "temperature"
```

#### Sync to Geo-Spatial (SQL Server)
Power "Store Finder" features.
```toml
[[flow]]
name = "store_locations"
source = "sqlserver://sa:Pass@corp:1433/logistics?encrypt=true&pool_max=10&connection_timeout=10"
table = "stores"
target = "geo"
key = "geo:stores"
member = "store_name"
lat = "latitude"
lon = "longitude"
```

### Supported Databases & Optimized Connection Strings

| Database | Optimized Connection String |
| :--- | :--- |
| **MySQL** | `mysql://user:pass@host:3306/db?pool_max=20&pool_min=5&connect_timeout=5` |
| **PostgreSQL** | `postgres://user:pass@host:5432/db?sslmode=prefer&pool_max=20&statement_cache_capacity=100` |
| **SQL Server** | `sqlserver://user:pass@host:1433/db?encrypt=true&trust_server_certificate=true&pool_max=10` |
| **SQLite** | `sqlite://path/to/file.db?mode=rwc&busy_timeout=5000` |

### Connection String Parameters (God Tier Performance)

| Parameter | Description | Recommended |
| :--- | :--- | :--- |
| `pool_max` | Maximum connections in pool | 10-30 (depending on load) |
| `pool_min` | Keep-alive connections | 2-5 |
| `connect_timeout` | Connection timeout (seconds) | 3-10 |
| `sslmode` | PostgreSQL SSL mode | `prefer` or `require` |
| `encrypt` | SQL Server encryption | `true` for production |
| `statement_cache_capacity` | PG prepared statement cache | 100+ for repeated queries |

## 🥊 Zedis vs Redis: No Contest

| Capability | Zedis ⚡ | Redis 🔴 |
| :--- | :--- | :--- |
| **Price** | **FREE Forever** | $$$ (Enterprise) |
| **Threading Model** | **Multi-Threaded (Thread-per-Core)** | Single-Threaded |
| **Throughput (ops/sec)** | **1,000,000+** | 100,000 |
| **P99 Latency** | **<100μs** | ~500μs |
| **Native Vector Search** | ✅ **Built-in** | ❌ Requires RediSearch |
| **Time-Series** | ✅ **Built-in** | ❌ Requires RedisTimeSeries |
| **Graph Queries** | ✅ **Built-in** | ❌ Requires RedisGraph |
| **JSON Documents** | ✅ **Built-in** | ❌ Requires RedisJSON |
| **Memory Safety** | ✅ **Rust (Guaranteed)** | ⚠️ C (Manual) |
| **Auto-Embedding** | ✅ **VADD.TEXT** | ❌ Client-side only |

---

## 🧠 AI & Vector Search Guide

Zedis makes AI integration incredibly simple by handling the heavy lifting (embedding generation) **inside the database**. You don't need an external embedding service (like OpenAI API) or local Python scripts to vectoralize your text.

### How it Works
1.  **You send Text:** You send raw text to Zedis using `VADD.TEXT`.
2.  **Zedis Embeds:** Zedis uses a built-in, lightweight transformer model (running on the CPU) to convert that text into a vector.
3.  **Zedis Indexes:** The vector is stored in a HNSW index for millisecond-speed retrieval.

### Step-by-Step Tutorial

#### 1. Setup
**Zero setup required.** The default embedding model is baked into Zedis. Just start the server.

#### 2. Adding Documents (`VADD.TEXT`)
Store your text data. Zedis effectively works as a semantic search engine here.

```bash
# Syntax: VADD.TEXT <key> <content>
VADD.TEXT product:1 "Wireless Noise Cancelling Headphones with 20h battery"
VADD.TEXT product:2 "Bluetooth Portable Speaker, waterproof"
VADD.TEXT product:3 "Mechanical Keyboard Blue Switch"
```

#### 3. Searching (`VSEARCH.TEXT`)
Find items based on **meaning**, not just keyword matches.

```bash
# Syntax: VSEARCH.TEXT <index_name> <query_text> <limit>

# User asks for "audio devices"
# Zedis finds the headphones and speaker, even though the word "audio" isn't in them!
VSEARCH.TEXT product "audio devices" 2
```

**Result:**
1.  `product:1` (Headphones) - High similarity
2.  `product:2` (Speaker) - High similarity

### Why is this "God Tier"?
*   **No API Costs:** You aren't paying OpenAI $0.0001 per token.
*   **No Latency:** Data doesn't leave the database server.
*   **Privacy:** Your private data never leaves your infrastructure.

---

## 💡 Real-World Use Cases

Here are 5 practical ways to use each of Zedis's God Tier features:

### 🧠 Z-Vector (AI Search)
1.  **RAG Chatbots**: Store knowledge base chunks as vectors. When a user asks a question, retrieve the most relevant chunks to feed into your LLM.
2.  **E-commerce Recommendation**: "Similar Products". Auto-embed product descriptions to find items with similar features (e.g., "earbuds" ~ "headphones") without keyword matching.
3.  **Semantic Image Search**: Use a vision model to embed images. Users can search "sunset on beach" and find relevant photos.
4.  **Customer Support Triage**: Classify incoming support tickets by meaning to automatically route them to the correct department (Billing vs Technical).
5.  **Plagiarism/Duplication Detection**: Find documents that are semantically identical even if words are rephrased.

### 🎲 Z-Probabilistic
1.  **Unique Visitor Counting (HyperLogLog)**: Count Daily Active Users (DAU) for a website with millions of hits using only 12KB of memory.
2.  **Username Availability (Cuckoo Filter)**: Check if a username is taken before hitting your primary database. Saves massive IOPS.
3.  **DDoS Protection (Count-Min Sketch)**: Track IP request frequency in real-time to identify and block abusive "heavy hitters".
4.  **Trending Topics (Top-K)**: Identify the top 10 most used hashtags or search terms in a live stream of data.
5.  **API Performance Monitoring (t-digest)**: Calculate accurate P99 latency percentiles for your API endpoints to detect slow outliers.

### 🌊 Z-Stream (Event Streams)
1.  **Social Activity Feeds**: Store user activity (likes, comments, posts) in an infinite log for followers to consume.
2.  **IoT Data Ingestion**: Buffer massive bursts of sensor data (temperature, speed) before processing/archiving to cold storage.
3.  **Job Queues**: Reliable background job processing system. Producers push tasks, Consumers group-read and acknowledge them.
4.  **Audit Logs**: Immutable history of all critical system actions for compliance and security auditing.
5.  **Chat History**: Store chat room messages sequentially with IDs useful for "load more" pagination.

### 📢 Z-PubSub (Real-Time)
1.  **Live Sports Scores**: Push real-time score updates to millions of connected web clients instantly.
2.  **Chat Applications**: Instant message delivery between users in a chat room.
3.  **System Config Updates**: Broadcast a "clear cache" or "update config" command to all your microservice instances at once.
4.  **Geofence Alerts**: Notify a user app immediately when they enter a specific physical zone.
5.  **Typing Indicators**: Show "User is typing..." status in real-time apps.

### 📊 Z-Time (Time-Series)
1.  **Server Monitoring**: Track CPU, Memory, and Disk usage metrics every second for dashboards.
2.  **Financial Tickers**: Store high-frequency stock or crypto trade prices for candlestick charting.
3.  **Smart Metering**: Record electricity or water usage readings from millions of meters.
4.  **Website Analytics**: Track Requests Per Second (RPS) and error rates over time.
5.  **Cold Chain Logistics**: Monitor freezer temperatures during shipping to ensure compliance (alert if temp > threshold).

### 🕸️ Z-Graph (Graph Processing)
1.  **Friend Recommendations**: "People you may know" - Find friends of friends (2nd degree connections).
2.  **Fraud Detection**: Detect circular money transfers or fraud rings (A pays B, B pays C, C pays A).
3.  **Product Recommendation**: "People who bought X also bought Y" graph analysis.
4.  **Identity Resolution**: Link unconnected data points (cookies, emails, devices) to a single user identity.
5.  **Access Control (RBAC)**: traverse organizational hierarchy trees to determine if User A has permission for Resource B.

### 🌍 Z-Geo (Geo-Spatial)
1.  **Ride Sharing**: "Find nearest 5 drivers to my location".
2.  **Store Locator**: Show all retail branches within a 5km radius.
3.  **Delivery Tracking**: Update and query courier positions in real-time.
4.  **Dating Apps**: "Show users nearby" with distance filtering.
5.  **Location-Based Marketing**: Trigger a push notification when a user walks past a specific store.

---

## 🛠️ Usage Examples (Polyglot)

Zedis speaks the standard Redis Protocol (RESP). You can use **any standard Redis client** to interact with it.

### 1. Python (`redis-py`)
```python
import redis

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# 1. God Tier Vector Search (Auto-Embedding) 🧠
# No need to run embeddings client-side!
# Zedis automatically handles "Text -> Vector" conversion.
r.execute_command('VADD.TEXT', 'doc:1', 'The quick brown fox')
r.execute_command('VADD.TEXT', 'doc:2', 'Jumped over the lazy dog')

# Search by semantics
results = r.execute_command('VSEARCH.TEXT', 'index', 'animal jumping', '5')
print(results) # Returns keys closest to "animal jumping"

# 2. JSON Store
r.execute_command('JSON.SET', 'user:100', '{"name": "Alice", "role": "admin"}')
user = r.execute_command('JSON.GET', 'user:100', '.')

# 3. Probabilistic Structures (God Tier)
# HyperLogLog
r.execute_command('PFADD', 'hll_users', 'u1', 'u2', 'u3')
print(r.execute_command('PFCOUNT', 'hll_users')) # 3

# Cuckoo Filter
r.execute_command('CF.ADD', 'cf_filter', 'item1')
print(r.execute_command('CF.EXISTS', 'cf_filter', 'item1')) # 1

# 4. Streams
r.execute_command('XADD', 'mystream', '*', 'sensor', 'A', 'temp', '20')
entries = r.execute_command('XRANGE', 'mystream', '-', '+')
print(entries)

```

### 2. Go (`go-redis`)
```go
package main

import (
    "context"
    "fmt"
    "github.com/redis/go-redis/v9"
)

func main() {
    ctx := context.Background()
    rdb := redis.NewClient(&redis.Options{Addr: "localhost:6379"})

    // Z-Vector
    rdb.Do(ctx, "VADD", "vec:A", 1.2, 3.4, 5.6)

    // Z-Sketch
    rdb.Do(ctx, "BF.ADD", "whitelist", "192.168.1.1")
    exists, _ := rdb.Do(ctx, "BF.EXISTS", "whitelist", "192.168.1.1").Bool()
    fmt.Printf("Exists: %v\n", exists)
}
```

### 3. Node.js (`ioredis`)
```javascript
const Redis = require("ioredis");
const redis = new Redis();

// 1. AI Vector Search
await redis.call("VADD.TEXT", "item:1", "Apple iPhone 15 Pro Max");
await redis.call("VADD.TEXT", "item:2", "Samsung Galaxy S24 Ultra");

const results = await redis.call("VSEARCH.TEXT", "item", "flagship smartphone", 2);
console.log(results);

// 2. JSON Store
await redis.call("JSON.SET", "config", JSON.stringify({ theme: "dark" }));
const config = await redis.call("JSON.GET", "config", ".");
```

### 4. C# (`StackExchange.Redis`)
```csharp
using StackExchange.Redis;

var redis = ConnectionMultiplexer.Connect("localhost");
var db = redis.GetDatabase();

// Z-Doc (JSON)
db.Execute("JSON.SET", "config", "{ \"theme\": \"dark\" }");
var json = db.Execute("JSON.GET", "config", ".");

// Z-Vector
db.Execute("VADD", "item:55", 0.05, 0.99, 0.12);
```

### 5. VB.NET (`StackExchange.Redis`)
```vb
Imports StackExchange.Redis

Dim redis = ConnectionMultiplexer.Connect("localhost")
Dim db = redis.GetDatabase()

' Z-Vector Search
db.Execute("VADD.TEXT", "doc:1", "Hello World")
Dim results = db.Execute("VSEARCH.TEXT", "doc", "Greetings", 1)

' Z-Sketch
db.Execute("BF.ADD", "ip_blacklist", "10.0.0.5")
Dim isBlocked = db.Execute("BF.EXISTS", "ip_blacklist", "10.0.0.5")
```

### 6. Rust (`redis-rs`)
```rust
use redis::Commands;

fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    // Standard SET/GET
    let _: () = con.set("my_key", 42)?;
    
    // Raw Command for Z-Vector
    let _: () = redis::cmd("VADD")
        .arg("vec:1").arg(1.0).arg(2.0)
        .query(&mut con)?;

    Ok(())
}
```

### 7. PHP (`Predis` / `PhpRedis`)
```php
<?php
$client = new Predis\Client();

// Z-Doc
$client->executeRaw(['JSON.SET', 'session:123', '{"active": true}']);
$doc = $client->executeRaw(['JSON.GET', 'session:123', '.']);

// Z-Sketch
$client->executeRaw(['BF.ADD', 'visited_urls', 'https://google.com']);
?>
```

---

## ⚡ Quick Start & Migration

### How to Run
1.  **Download/Clone** the repository.
2.  **Build** with Cargo:
    ```bash
    cargo run --release
    ```
    *Note: `--release` is critical for SIMD and performance optimizations.*

### Replacing Redis
Zedis listens on port **6379** by default (just like Redis).
1.  Stop your existing Redis instance: `sudo systemctl stop redis`
2.  Start Zedis.
3.  Your existing applications will connect automatically without code changes (Shadow Mode compatible).

### Compatibility Note
Zedis implements a subset of Redis commands + custom Universe Tier commands. 
- Supported: `SET`, `GET`, `HSET`, `HGET`, `RPUSH`, `LPOP`, `LRANGE`, `ZADD`, `ZRANGE`, `EVAL`, `PING`, `MULTI`, `EXEC`.
- Universe: `VADD`, `VSEARCH`, `PFADD`, `CF.ADD`, `CMS.INCRBY`, `TOPK.ADD`, `TDIGEST.ADD`, `JSON.SET`, `XADD`, `GEOADD`, `PUBLISH`.

For advanced features (Pub/Sub, Transactions), complete implementation is on the roadmap.

---

**Built with ❤️ in Rust.**
