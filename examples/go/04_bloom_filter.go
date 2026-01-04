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

	fmt.Println("--- 04 Bloom Filter ---")
	key := "spam_emails"

	// BF.ADD
	rdb.Do(ctx, "BF.ADD", key, "spam@evil.com")
	rdb.Do(ctx, "BF.ADD", key, "virus@bad.com")

	// BF.EXISTS
	check := "spam@evil.com"
	exists, _ := rdb.Do(ctx, "BF.EXISTS", key, check).Int()

	fmt.Printf("Is '%s' blocked? %v\n", check, exists == 1)

	checkSafe := "friend@good.com"
	existsSafe, _ := rdb.Do(ctx, "BF.EXISTS", key, checkSafe).Int()
	fmt.Printf("Is '%s' blocked? %v\n", checkSafe, existsSafe == 1)
}
