const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

async function main() {
    console.log("--- 02 AI Vector Search (Auto-Embedding) ---");

    // 1. Ingest Data
    const products = [
        { id: "prod:1", desc: "Smartphone with 5G and OLED screen" },
        { id: "prod:2", desc: "Laptop 16GB RAM SSD High Performance" },
        { id: "prod:3", desc: "Wireless Earbuds Noise Cancelling" },
    ];

    console.log("Ingesting products...");
    for (const p of products) {
        // VADD.TEXT <key> <text>
        await redis.call("VADD.TEXT", p.id, p.desc);
        console.log(`Added ${p.id}`);
    }

    // 2. Semantic Search
    const query = "computer fast memory";
    console.log(`\nSearching for: '${query}'`);

    // VSEARCH.TEXT <index> <query> <limit>
    const results = await redis.call("VSEARCH.TEXT", "prod", query, 2);
    console.log("Results (Top 2):", results);
    // Expected: prod:2 should be top result

    redis.disconnect();
}

main();
