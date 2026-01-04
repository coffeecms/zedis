const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

async function main() {
    console.log("--- 04 Bloom Filter ---");
    const key = "crawled_urls";

    const urls = [
        "https://google.com",
        "https://github.com",
        "https://rust-lang.org",
    ];

    // BF.ADD
    for (const url of urls) {
        await redis.call("BF.ADD", key, url);
    }
    console.log("Added 3 URLs to Bloom Filter.");

    // BF.EXISTS
    const checkUrl = "https://google.com";
    const exists = await redis.call("BF.EXISTS", key, checkUrl); // Returns 1 or 0

    console.log(`Does '${checkUrl}' exist? ${exists === 1 ? "Yes" : "No"}`);

    const unknownUrl = "https://unknown-site.com";
    const exists2 = await redis.call("BF.EXISTS", key, unknownUrl);
    console.log(`Does '${unknownUrl}' exist? ${exists2 === 1 ? "Yes" : "No"}`);

    redis.disconnect();
}

main();
