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

    fmt.Println("--- 01 Basic Key-Value Operations ---")

    // SET
    fmt.Println("Setting key 'language' to 'Go'...")
    err := rdb.Set(ctx, "language", "Go", 0).Err()
    if err != nil {
        panic(err)
    }

    // GET
    val, err := rdb.Get(ctx, "language").Result()
    if err != nil {
        panic(err)
    }
    fmt.Println("language:", val)

    // EXPIRY
    fmt.Println("Setting key 'temp' with 5s expiry...")
    rdb.Set(ctx, "temp", "gone_soon", 5*time.Second)
    
    // PING
    pong, _ := rdb.Ping(ctx).Result()
    fmt.Println("PING:", pong)
}
