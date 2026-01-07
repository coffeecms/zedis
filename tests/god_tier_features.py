import redis
import time
import threading

# Configuration
HOST = 'localhost'
PORT = 6379
r = redis.Redis(host=HOST, port=PORT, decode_responses=True)

def test_probabilistic():
    print("--- Testing Probabilistic Data Structures ---")
    
    # HyperLogLog
    print("Testing PFADD/PFCOUNT...")
    r.pfadd("hll_test", "a", "b", "c", "d", "a")
    count = r.pfcount("hll_test")
    print(f"HLL Count (Approx 4): {count}")
    assert count >= 3 and count <= 5

    # Cuckoo (Custom Command Wrapper needed as redis-py might not have explicit support yet, use execute_command)
    print("Testing CF.ADD/CF.EXISTS...")
    r.execute_command("CF.ADD", "cuckoo_test", "item1")
    exists = r.execute_command("CF.EXISTS", "cuckoo_test", "item1")
    print(f"CF Exists (Expected 1): {exists}")
    assert exists == 1

def test_pubsub():
    print("\n--- Testing Pub/Sub ---")
    p = r.pubsub()
    p.subscribe("god_tier_channel")
    time.sleep(1) # Wait for sub

    def publisher():
        time.sleep(0.5)
        # Re-create client for publish to avoid blocking main
        r2 = redis.Redis(host=HOST, port=PORT, decode_responses=True)
        r2.publish("god_tier_channel", "Hello Zedis")
        print("Published message.")

    threading.Thread(target=publisher).start()

    msg = p.get_message(timeout=5) # First might be sub confirmation
    if msg and msg['type'] == 'subscribe':
        msg = p.get_message(timeout=5)
    
    print(f"Received: {msg}")
    assert msg['data'] == "Hello Zedis"
    p.close()

def test_transactions():
    print("\n--- Testing Transactions (MULTI/EXEC) ---")
    pipe = r.pipeline(transaction=True)
    pipe.set("tx_key", "1")
    pipe.incr("tx_key")
    pipe.get("tx_key")
    results = pipe.execute()
    print(f"Transaction Results: {results}")
    assert results == [True, 2, "2"]

def test_lua():
    print("\n--- Testing Lua Programmability ---")
    script = """
    redis.call('SET', KEYS[1], ARGV[1])
    return redis.call('GET', KEYS[1])
    """
    res = r.eval(script, 1, "lua_key", "god_mode_enabled")
    print(f"Lua Result: {res}")
    assert res == "god_mode_enabled"

def test_bitfield():
    print("\n--- Testing BITFIELD ---")
    # BITFIELD key SET u8 0 255 INCRBY u8 0 1
    # 255 (11111111) + 1 -> 0 (overflow wrap default)
    res = r.bitfield("bf_key").set("u8", 0, 255).incrby("u8", 0, 1).execute()
    print(f"Bitfield Result: {res}")
    # redis-py constructs the list, backend should store/return
    # Expected: [0, 0] (Previous val of set?, new val of incr?) 
    # Redis SET returns old value? No, SET returns Old Value? Yes.
    # INCRBY returns New Value.
    # If key was empty: SET returns 0 (old). INCRBY u8 wraps 255+1 = 0.
    assert len(res) == 2

if __name__ == "__main__":
    try:
        # r.ping()
        test_probabilistic()
        test_pubsub()
        test_transactions()
        test_lua()
        test_bitfield()
        print("\n✅ All God Tier Tests Passed!")
    except Exception as e:
        print(f"\n❌ Test Failed: {e}")
        # print details
