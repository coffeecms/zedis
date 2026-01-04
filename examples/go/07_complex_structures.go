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

	fmt.Println("--- 07 Complex Structures ---")

	// Sort Set (ZSET)
	rdb.ZAdd(ctx, "leaderboard", redis.Z{Score: 100, Member: "Player1"})
	rdb.ZAdd(ctx, "leaderboard", redis.Z{Score: 200, Member: "Player2"})

	rank, _ := rdb.ZRevRank(ctx, "leaderboard", "Player2").Result()
	fmt.Printf("Player2 Rank: %d (0-indexed)\n", rank)

	// Hash
	rdb.HSet(ctx, "config:app", "max_users", "1000", "timeout", "30s")
	conf, _ := rdb.HGetAll(ctx, "config:app").Result()
	fmt.Println("Config:", conf)
}
