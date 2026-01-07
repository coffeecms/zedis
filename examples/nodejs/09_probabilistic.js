const Redis = require("ioredis");
const redis = new Redis({ host: "localhost", port: 6379 });

async function run() {
    console.log("--- 09 God Tier Probabilistic Data Structures ---");

    // 1. HyperLogLog
    console.log("\n[HyperLogLog]");
    await redis.call("PFADD", "hll_node", "user_a", "user_b");
    const pfCount = await redis.call("PFCOUNT", "hll_node");
    console.log(`Estimated Count: ${pfCount}`);

    // 2. Cuckoo Filter
    console.log("\n[Cuckoo Filter]");
    await redis.call("CF.ADD", "cf_node", "item1");
    const exists = await redis.call("CF.EXISTS", "cf_node", "item1");
    console.log(`Item1 Exists: ${exists}`);

    // 3. Count-Min Sketch
    console.log("\n[Count-Min Sketch]");
    await redis.call("CMS.INCRBY", "cms_node", "click", 50);
    const clicks = await redis.call("CMS.QUERY", "cms_node", "click");
    console.log(`Clicks (Approx): ${clicks}`);

    // 4. Top-K
    console.log("\n[Top-K]");
    await redis.call("TOPK.ADD", "topk_node", "x", "y", "z", "x", "x");
    const top = await redis.call("TOPK.LIST", "topk_node");
    console.log("Top K List:", top);

    // 5. t-digest
    console.log("\n[t-digest]");
    await redis.call("TDIGEST.ADD", "td_node", 1.5);
    await redis.call("TDIGEST.ADD", "td_node", 2.5);
    const median = await redis.call("TDIGEST.QUANTILE", "td_node", 0.5);
    console.log(`Median values: ${median}`);

    redis.disconnect();
}

run().catch(console.error);
