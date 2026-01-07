package main

import (
	"context"
	"fmt"

	"github.com/redis/go-redis/v9"
)

func main() {
	ctx := context.Background()
	rdb := redis.NewClient(&redis.Options{Addr: "localhost:6379"})

	fmt.Println("--- 11 Transactions ---")

	pipe := rdb.TxPipeline()

	pipe.Set(ctx, "tx_key", "step1", 0)
	pipe.Incr(ctx, "tx_counter")

	cmds, err := pipe.Exec(ctx)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	}

	for _, cmd := range cmds {
		fmt.Printf("Result: %v\n", cmd)
	}
}
