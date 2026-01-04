package main

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

var ctx = context.Background()

func main() {
	rdb := redis.NewClient(&redis.Options{
		Addr: "localhost:6379",
	})

	fmt.Println("--- 05 Time Series ---")
	key := "stock:AAPL"

	now := time.Now().UnixMilli()

	// TS.ADD
	rdb.Do(ctx, "TS.ADD", key, now, 150.50)
	rdb.Do(ctx, "TS.ADD", key, now+1000, 151.00)
	rdb.Do(ctx, "TS.ADD", key, now+2000, 150.75)

	// TS.RANGE
	fmt.Println("Querying stock data...")
	res, _ := rdb.Do(ctx, "TS.RANGE", key, now, now+5000).Result()
	fmt.Println(res)
}
