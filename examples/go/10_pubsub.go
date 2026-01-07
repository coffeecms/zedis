package main

import (
	"context"
	"fmt"
	"time"
	"github.com/redis/go-redis/v9"
)

func main() {
	ctx := context.Background()
	rdb := redis.NewClient(&redis.Options{Addr: "localhost:6379"})

	fmt.Println("--- 10 Pub/Sub ---")

	pubsub := rdb.Subscribe(ctx, "go_channel")
	defer pubsub.Close()

	// Subscriber goroutine
	go func() {
		ch := pubsub.Channel()
		for msg := range ch {
			fmt.Printf("[Sub] Received: %s\n", msg.Payload)
			if msg.Payload == "quit" {
				return
			}
		}
	}()

	time.Sleep(100 * time.Millisecond)

	// Publish
	rdb.Publish(ctx, "go_channel", "Hello from Go")
	time.Sleep(100 * time.Millisecond)
	rdb.Publish(ctx, "go_channel", "quit")

	time.Sleep(500 * time.Millisecond)
}
