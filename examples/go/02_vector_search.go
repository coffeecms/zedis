package main

import (
	"context"
	"fmt"

	"github.com/redis/go-redis/v9"
)

var ctx = context.Background()

func main() {
	rdb := redis.NewClient(&redis.Options{
		Addr: "localhost:6379",
	})

	fmt.Println("--- 02 AI Vector Search (Auto-Embedding) ---")

	// 1. Ingest
	fmt.Println("Ingesting headlines...")
	rdb.Do(ctx, "VADD.TEXT", "news:1", "Tech giants release new AI models")
	rdb.Do(ctx, "VADD.TEXT", "news:2", "Stock market hits all time high")
	rdb.Do(ctx, "VADD.TEXT", "news:3", "Local weather forecast: Sunny and warm")

	// 2. Search
	query := "artificial intelligence updates"
	fmt.Printf("\nSearching for: '%s'\n", query)

	// VSEARCH.TEXT <prefix> <query> <limit>
	res, err := rdb.Do(ctx, "VSEARCH.TEXT", "news", query, 2).Result()
	if err != nil {
		panic(err)
	}

	fmt.Println("Results:", res)
	// Expected: news:1 should be top match
}
