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

	fmt.Println("--- 06 Graph Processing ---")
	key := "family_tree"

	// GRAPH.ADD Parent -> Child
	rdb.Do(ctx, "GRAPH.ADD", key, "Grandpa", "Dad")
	rdb.Do(ctx, "GRAPH.ADD", key, "Dad", "Son")

	// GRAPH.BFS
	fmt.Println("Descendants of Grandpa (Depth 2):")
	res, _ := rdb.Do(ctx, "GRAPH.BFS", key, "Grandpa", 2).Result()
	fmt.Println(res)
}
