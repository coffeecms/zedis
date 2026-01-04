const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

async function main() {
    console.log("--- 03 JSON Document Store ---");

    const config = {
        theme: "dark",
        notifications: {
            email: true,
            sms: false,
        },
        max_items: 50,
    };

    // JSON.SET
    console.log("Saving config...");
    await redis.call("JSON.SET", "user:config:1", JSON.stringify(config));

    // JSON.GET
    console.log("Retrieving config...");
    const rawData = await redis.call("JSON.GET", "user:config:1", ".");
    const storedConfig = JSON.parse(rawData);
    console.log("Stored Config:", storedConfig);

    console.log(`Theme is: ${storedConfig.theme}`);

    redis.disconnect();
}

main();
