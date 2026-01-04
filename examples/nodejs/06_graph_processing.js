const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

async function main() {
    console.log("--- 06 Graph Processing ---");
    const key = "routes";

    // Graph: City connections
    // NYC -> London -> Paris
    // NYC -> Tokyo

    console.log("Building route graph...");
    await redis.call("GRAPH.ADD", key, "NYC", "London");
    await redis.call("GRAPH.ADD", key, "London", "Paris");
    await redis.call("GRAPH.ADD", key, "NYC", "Tokyo");

    // BFS
    console.log("Finding destinations from NYC (max 2 stops)...");
    const destinations = await redis.call("GRAPH.BFS", key, "NYC", 2);
    console.log("Destinations:", destinations);

    redis.disconnect();
}

main();
