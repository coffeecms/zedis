const Redis = require("ioredis");
const redis = new Redis({ host: "localhost", port: 6379 });

async function run() {
    console.log("--- 11 Transactions ---");

    const key1 = "node:tx:1";
    const key2 = "node:tx:2";

    await redis.psetex(key1, 10000, "100");
    await redis.psetex(key2, 10000, "200");

    console.log("Beginning Transaction...");
    const pipeline = redis.multi();

    pipeline.incrby(key1, 10);
    pipeline.decrby(key2, 10);

    // Note: ioredis 'multi' abstraction handles MULTI/EXEC automatically when .exec() is called
    const results = await pipeline.exec();

    console.log("Transaction Results:");
    // Results is an array of [error, result] tuples
    results.forEach((res, idx) => {
        console.log(`Cmd ${idx + 1}: Error=${res[0]}, Value=${res[1]}`);
    });

    const v1 = await redis.get(key1);
    const v2 = await redis.get(key2);
    console.log(`Final Values: ${v1}, ${v2}`);

    redis.disconnect();
}

run().catch(console.error);
