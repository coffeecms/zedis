const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

async function main() {
    console.log("--- 07 Complex Structures ---");

    // Lists
    await redis.rpush("queue:jobs", "job1", "job2", "job3");
    const job = await redis.lpop("queue:jobs");
    console.log(`Processed: ${job}`);

    // Sets
    await redis.sadd("tags", "frontend", "javascript", "nodejs");
    const isMember = await redis.sismember("tags", "rust");
    console.log(`Is 'rust' a tag? ${isMember}`);

    // Hash
    await redis.hset("user:session:99", {
        "ip": "127.0.0.1",
        "last_login": Date.now()
    });
    const ip = await redis.hget("user:session:99", "ip");
    console.log(`Session IP: ${ip}`);

    redis.disconnect();
}

main();
