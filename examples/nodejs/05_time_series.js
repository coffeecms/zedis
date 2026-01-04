const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

async function main() {
    console.log("--- 05 Time Series ---");
    const key = "server:cpu:load";

    const now = Date.now();

    // TS.ADD
    console.log("Adding CPU load metrics...");
    await redis.call("TS.ADD", key, now, 45.0);
    await redis.call("TS.ADD", key, now + 1000, 50.5);
    await redis.call("TS.ADD", key, now + 2000, 48.2);

    // TS.RANGE
    console.log("Querying metrics...");
    const data = await redis.call("TS.RANGE", key, now, now + 5000);
    console.log("Data:", data);

    redis.disconnect();
}

main();
