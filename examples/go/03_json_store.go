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

	fmt.Println("--- 03 JSON Document Store ---")

	jsonData := `{"name": "Gopher", "type": "mascot", "skills": ["concurrency"]}`

	// JSON.SET
	fmt.Println("Storing JSON...")
	rdb.Do(ctx, "JSON.SET", "mascot:1", jsonData)

	// JSON.GET
	fmt.Println("Retrieving JSON...")
	val, err := rdb.Do(ctx, "JSON.GET", "mascot:1", ".").Result()
	if err != nil {
		panic(err)
	}
	fmt.Println("Data:", val)
}
