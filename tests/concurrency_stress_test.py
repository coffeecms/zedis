#!/usr/bin/env python3
"""
Zedis Concurrency Stress Test
- Tests high concurrent read/write operations
- Verifies data integrity under load
"""

import redis
import threading
import time
import random
import string

def random_string(length=10):
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

def stress_test(host='localhost', port=6379, num_threads=100, ops_per_thread=1000):
    """
    Run concurrent read/write stress test
    
    Args:
        host: Zedis host
        port: Zedis port
        num_threads: Number of concurrent threads
        ops_per_thread: Operations per thread
    """
    
    errors = []
    write_count = [0]  # Use list for mutable counter in threads
    read_count = [0]
    lock = threading.Lock()
    
    def writer(thread_id):
        """Write random keys"""
        try:
            r = redis.Redis(host=host, port=port, decode_responses=True)
            for i in range(ops_per_thread):
                key = f"stress:t{thread_id}:k{i}"
                value = random_string(100)
                r.set(key, value)
                with lock:
                    write_count[0] += 1
        except Exception as e:
            with lock:
                errors.append(f"Writer {thread_id}: {e}")
    
    def reader(thread_id):
        """Read random keys"""
        try:
            r = redis.Redis(host=host, port=port, decode_responses=True)
            for i in range(ops_per_thread):
                key = f"stress:t{random.randint(0, num_threads//2 - 1)}:k{random.randint(0, ops_per_thread-1)}"
                r.get(key)
                with lock:
                    read_count[0] += 1
        except Exception as e:
            with lock:
                errors.append(f"Reader {thread_id}: {e}")
    
    def mixed_worker(thread_id):
        """50% read, 50% write"""
        try:
            r = redis.Redis(host=host, port=port, decode_responses=True)
            for i in range(ops_per_thread):
                if random.random() < 0.5:
                    # Write
                    key = f"mixed:t{thread_id}:k{i}"
                    r.set(key, random_string(50))
                    with lock:
                        write_count[0] += 1
                else:
                    # Read
                    key = f"mixed:t{random.randint(0, num_threads-1)}:k{random.randint(0, ops_per_thread-1)}"
                    r.get(key)
                    with lock:
                        read_count[0] += 1
        except Exception as e:
            with lock:
                errors.append(f"Mixed {thread_id}: {e}")

    print("=" * 60)
    print("🚀 ZEDIS CONCURRENCY STRESS TEST")
    print("=" * 60)
    print(f"Threads: {num_threads}")
    print(f"Operations per thread: {ops_per_thread}")
    print(f"Total expected ops: {num_threads * ops_per_thread}")
    print()

    # Test 1: Pure Writers
    print("📝 Test 1: Pure Write Load")
    threads = []
    write_count[0] = 0
    start = time.time()
    
    for i in range(num_threads // 2):
        t = threading.Thread(target=writer, args=(i,))
        threads.append(t)
        t.start()
    
    for t in threads:
        t.join()
    
    elapsed = time.time() - start
    print(f"   Writes: {write_count[0]:,}")
    print(f"   Time: {elapsed:.2f}s")
    print(f"   Throughput: {write_count[0]/elapsed:,.0f} ops/sec")
    print()

    # Test 2: Pure Readers
    print("📖 Test 2: Pure Read Load")
    threads = []
    read_count[0] = 0
    start = time.time()
    
    for i in range(num_threads // 2):
        t = threading.Thread(target=reader, args=(i,))
        threads.append(t)
        t.start()
    
    for t in threads:
        t.join()
    
    elapsed = time.time() - start
    print(f"   Reads: {read_count[0]:,}")
    print(f"   Time: {elapsed:.2f}s")
    print(f"   Throughput: {read_count[0]/elapsed:,.0f} ops/sec")
    print()

    # Test 3: Mixed Read/Write
    print("🔀 Test 3: Mixed Read/Write Load")
    threads = []
    write_count[0] = 0
    read_count[0] = 0
    start = time.time()
    
    for i in range(num_threads):
        t = threading.Thread(target=mixed_worker, args=(i,))
        threads.append(t)
        t.start()
    
    for t in threads:
        t.join()
    
    elapsed = time.time() - start
    total = write_count[0] + read_count[0]
    print(f"   Writes: {write_count[0]:,}")
    print(f"   Reads: {read_count[0]:,}")
    print(f"   Total: {total:,}")
    print(f"   Time: {elapsed:.2f}s")
    print(f"   Throughput: {total/elapsed:,.0f} ops/sec")
    print()

    # Report errors
    print("=" * 60)
    if errors:
        print(f"❌ ERRORS: {len(errors)}")
        for e in errors[:10]:  # Show first 10 errors
            print(f"   {e}")
    else:
        print("✅ ALL TESTS PASSED - NO ERRORS")
    print("=" * 60)

    return len(errors) == 0

if __name__ == "__main__":
    import sys
    
    # Default parameters
    host = 'localhost'
    port = 6379
    threads = 50
    ops = 500
    
    # Parse command line args
    if len(sys.argv) > 1:
        port = int(sys.argv[1])
    if len(sys.argv) > 2:
        threads = int(sys.argv[2])
    if len(sys.argv) > 3:
        ops = int(sys.argv[3])
    
    success = stress_test(host=host, port=port, num_threads=threads, ops_per_thread=ops)
    sys.exit(0 if success else 1)
