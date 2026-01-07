package main

import (
	"context"
	"fmt"
	"github.com/redis/go-redis/v9"
)

func main() {
	ctx := context.Background()
	rdb := redis.NewClient(&redis.Options{Addr: "localhost:6379"})

	fmt.Println("--- 09 God Tier Probabilistic Data Structures ---")

	// 1. HyperLogLog
	rdb.Do(ctx, "PFADD", "hll_go", "u1", "u2", "u3")
	count, _ := rdb.Do(ctx, "PFCOUNT", "hll_go").Int()
	fmt.Printf("[HLL] Count: %d\n", count)

	// 2. Cuckoo Filter
	rdb.Do(ctx, "CF.ADD", "cf_go", "apple")
	exists, _ := rdb.Do(ctx, "CF.EXISTS", "cf_go", "apple").Int()
	fmt.Printf("[Cuckoo] Exists: %d\n", exists)

	// 3. Count-Min Sketch
	rdb.Do(ctx, "CMS.INCRBY", "cms_go", "view", 10)
	val, _ := rdb.Do(ctx, "CMS.QUERY", "cms_go", "view").Int()
	fmt.Printf("[CMS] Count: %d\n", val)

	// 4. Top-K
	rdb.Do(ctx, "TOPK.ADD", "topk_go", "A", "B", "A", "C", "A")
	list, _ := rdb.Do(ctx, "TOPK.LIST", "topk_go").StringSlice()
	fmt.Printf("[TopK] List: %v\n", list)

	// 5. t-digest
	rdb.Do(ctx, "TDIGEST.ADD", "td_go", 50.5)
	rdb.Do(ctx, "TDIGEST.ADD", "td_go", 100.0)
	q, _ := rdb.Do(ctx, "TDIGEST.QUANTILE", "td_go", 0.5).Float64()
	fmt.Printf("[t-digest] Median: %.2f\n", q)
}
