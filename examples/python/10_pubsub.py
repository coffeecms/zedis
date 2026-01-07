import redis
import threading
import time

def subscriber_thread():
    r_sub = redis.Redis(host='localhost', port=6379, decode_responses=True)
    p = r_sub.pubsub()
    p.subscribe('news_channel')
    
    print("Subscribed to 'news_channel'. Waiting for messages...")
    
    # Listen for messages
    for message in p.listen():
        if message['type'] == 'message':
            print(f"[Subscriber] Received: {message['data']}")
            if message['data'] == 'STOP':
                break

print("--- 10 Pub/Sub ---")

# Start subscriber in background
t = threading.Thread(target=subscriber_thread)
t.start()

time.sleep(1) # Wait for subscription

# Publisher
r_pub = redis.Redis(host='localhost', port=6379, decode_responses=True)
print("[Publisher] Sending Hello...")
r_pub.publish('news_channel', 'Hello Zedis!')

time.sleep(0.5)

print("[Publisher] Sending World...")
r_pub.publish('news_channel', 'World is fast!')

time.sleep(0.5)
r_pub.publish('news_channel', 'STOP')

t.join()
print("Pub/Sub Demo Finished.")
